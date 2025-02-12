use fake::{
    Dummy, Fake,
    faker::internet::{en::Password, en::SafeEmail},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use validator::Validate;
#[derive(Clone, FromRow, Serialize, Debug)]
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
/// let todo_changeset: TodoChangeset = Faker.fake();
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
