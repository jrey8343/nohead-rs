use async_trait::async_trait;
use serde::de::DeserializeOwned;
use sqlx::{Sqlite, prelude::FromRow, sqlite::SqliteRow as DbRow};
use validator::Validate;

use crate::Error;

pub mod todo;

/// ------------------------------------------------------------------------
/// # An Entity trait to implement common CRUD methods on a database table
/// ------------------------------------------------------------------------
///
/// Implement the Model trait on a specific model to get a full set
/// of common CRUD functions: list, show, create, update, delete
///
/// # Example
///
/// ```rust
/// #[async_trait]
/// impl Entity for Person {
///     type Id = i64;
///     type Record: Person;
///     type Changeset: PersonChangeset;
///
///     async fn list(db_pool: &DbPool) -> Result<Vec<Self::Record<'_>>, Error> {
///         // your implementation here
///         Ok(vec![])
///         }
///     // ...other methods
/// ```
///
/// ------------------------------------------------------------------------
#[async_trait]
pub trait Entity {
    type Id: PartialOrd;
    type Record<'a>: FromRow<'a, DbRow>;
    type Changeset: Validate + DeserializeOwned;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Self::Record<'a>>, Error>;

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Option<Self::Record<'a>>, Error>;

    async fn create<'a>(
        record: Self::Changeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;

    async fn update<'a>(
        id: Self::Id,
        record: Self::Changeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;
}
