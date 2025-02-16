use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, prelude::FromRow};
use validator::Validate;

#[cfg(feature = "test-helpers")]
use fake::{
    Dummy, Fake,
    faker::internet::{en::Password, en::SafeEmail},
};

use super::Entity;
use crate::{Error, ResultExt, transaction};

#[derive(Clone, FromRow, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
}

/// A changeset representing the data that is intended to be used to either create a new user or update an existing user.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let user_changeset: UserChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct UserChangeset {
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
    #[validate(must_match(other = "password", message = "passwords do not match"))]
    pub confirm_password: String,
}

/// ------------------------------------------------------------------------
/// Authentication specific structs.
/// ------------------------------------------------------------------------
#[derive(Clone)]
pub struct LoggedInUser {
    pub email: String,
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
impl Dummy<UserChangeset> for UserChangeset {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &UserChangeset, rng: &mut R) -> Self {
        let password: String = Password(8..16).fake_with_rng(rng);
        Self {
            email: SafeEmail().fake_with_rng(rng),
            password: password.clone(),
            confirm_password: password,
        }
    }
}

/// ------------------------------------------------------------------------
/// Specific authentication related queries for the User entity.
/// ------------------------------------------------------------------------
impl User {
    pub async fn get_by_email(email: &str, pool: &SqlitePool) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
           select * from users where email = ?

"#,
            email
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}

/// ------------------------------------------------------------------------
/// Generic CRUD related queries for the User entity.
/// ------------------------------------------------------------------------
#[async_trait]
impl Entity for User {
    type Id = i64;

    type Record<'a> = User;

    type Changeset = UserChangeset;
    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Self::Record<'a>>, Error> {
        let users = sqlx::query_as!(
            User,
            r#"select id, email, password_hash from users

"#
        )
        .fetch_all(executor)
        .await?;

        Ok(users)
    }

    async fn load<'a>(
        id: i64,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"select id, email, password_hash from users where id = ?

"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(user)
    }

    async fn create<'a>(
        user: UserChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<User, Error> {
        user.validate()?;

        let user = sqlx::query_as!(
            User,
            r#"
            insert into users (email, password_hash)
            values (?, ?)
            returning *

"#,
            user.email,
            user.password
        )
        .fetch_one(executor)
        .await
        .map_constraint_err()?; // return an app error if user already exists

        Ok(user)
    }

    async fn create_batch(
        users: Vec<UserChangeset>,
        db_pool: &SqlitePool,
    ) -> Result<Vec<User>, Error> {
        let mut tx = transaction(db_pool).await?;

        let mut results: Vec<Self::Record<'_>> = vec![];

        for user in users {
            user.validate()?;

            let result = Self::create(user, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: i64,
        user: UserChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<User, Error> {
        user.validate()?;

        todo!("work out how to update email and password_hash");
        let user = sqlx::query_as!(
            User,
            r#"update users set email = (?) where id = (?) returning id, email, password_hash

"#,
            user.email,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(user)
    }

    async fn delete<'a>(
        id: i64,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"delete from users where id = ? returning id, email, password_hash

"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(user)
    }
    async fn delete_batch(ids: Vec<Self::Id>, db_pool: &SqlitePool) -> Result<Vec<User>, Error> {
        let mut tx = transaction(db_pool).await?;

        let mut results: Vec<Self::Record<'_>> = vec![];

        for id in ids {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
