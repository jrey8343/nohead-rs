use argon2::{
    Argon2, PasswordHasher,
    password_hash::{self, SaltString, rand_core::OsRng},
};
use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Type, prelude::FromRow};
use validator::Validate;

#[cfg(feature = "test-helpers")]
use fake::{
    Dummy, Fake, Faker,
    faker::internet::{en::Password, en::SafeEmail},
};

use crate::{Error, ResultExt};

#[derive(Clone, FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub status: UserStatus,
}

#[derive(Clone, Deserialize, Serialize, Debug, Type)]
#[sqlx(type_name = "TEXT")]
pub enum UserStatus {
    Confirmed,
    Pending,
}

impl From<String> for UserStatus {
    fn from(val: String) -> Self {
        match val.as_str() {
            "confirmed" => UserStatus::Confirmed,
            "pending" => UserStatus::Pending,
            _ => UserStatus::Pending,
        }
    }
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("password", &"[redacted]")
            .field("status", &self.status)
            .finish()
    }
}

/// RegisterUser is a changeset for creating a new user.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let user: RegisterUser = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone, Debug)]
#[cfg_attr(feature = "test-helpers", derive(serde::Serialize))]
pub struct RegisterUser {
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
    #[validate(must_match(other = "password", message = "passwords do not match"))]
    pub confirm_password: String,
}
/// ------------------------------------------------------------------------
/// Manual impl Dummy to allow re-use of the password in the confirm_password field.
/// ------------------------------------------------------------------------
///
/// Only used when the `test-helpers` feature is enabled.
///
/// # Returns
///
/// A dummy UserChangeset with a random email, password and confirm_password.
/// ------------------------------------------------------------------------
#[cfg(feature = "test-helpers")]
impl Dummy<Faker> for RegisterUser {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let password: String = Password(8..16).fake_with_rng(rng);
        Self {
            email: SafeEmail().fake_with_rng(rng),
            password: password.clone(),
            confirm_password: password,
        }
    }
}

/// UserCredentials is a changeset for logging in a user.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let creds: UserCredentials = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone, Debug)]
#[cfg_attr(feature = "test-helpers", derive(serde::Serialize))]
pub struct UserCredentials {
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
    pub next: Option<String>,
}

/// ------------------------------------------------------------------------
/// Authentication specific implementations for axum_login.
/// ------------------------------------------------------------------------
impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
        // We use the password hash as the auth
        // hash--what this means
        // is when the user changes their password the
        // auth session becomes invalid.
    }
}

impl User {
    pub async fn try_get_by_email(
        email: &str,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Option<User>, Error> {
        let user = sqlx::query_as!(
            User,
            r#"select * from users where email = ?

"#,
            email
        )
        .fetch_optional(executor)
        .await?;
        Ok(user)
    }

    pub async fn try_get_by_id(
        id: &i64,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Option<User>, Error> {
        let user = sqlx::query_as!(
            User,
            r#"select * from users where id = ?

"#,
            id
        )
        .fetch_optional(executor)
        .await?;

        Ok(user)
    }

    pub async fn create(
        user: RegisterUser,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<User, Error> {
        user.validate()?;

        let password_hash = generate_password_hash(&user.password)?;

        let user = sqlx::query_as!(
            User,
            r#"
            insert into users (email, password_hash, status)
            values (?, ?, ?)
            returning *

"#,
            user.email,
            password_hash,
            UserStatus::Pending
        )
        .fetch_one(executor)
        .await
        .map_constraint_err()?; // return an app error if user already exists

        Ok(user)
    }

    pub async fn update_status(
        id: i64,
        status: UserStatus,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"update users set status = (?) where id = (?) returning *

"#,
            status,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(user)
    }
}

/// ------------------------------------------------------------------------
/// Helper function to generate a password hash using argon2.
/// ------------------------------------------------------------------------
/// # Returns
///
/// A hashed password string.
/// ------------------------------------------------------------------------
fn generate_password_hash(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hashed_password)
}
