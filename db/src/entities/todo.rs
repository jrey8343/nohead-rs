#[cfg(feature = "test-helpers")]
use fake::{Dummy, faker::lorem::en::*};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, prelude::FromRow};
use validator::Validate;

use super::Entity;
use crate::{Error, transaction};

/// A todo item.
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct Todo {
    /// The id of the record.
    pub id: String,
    /// The description, i.e. what to do.
    pub description: String,
}

/// A changeset representing the data that is intended to be used to either create a new task or update an existing task.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let todo_changeset: TodoChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct TodoChangeset {
    /// The description must be at least 1 character long.
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
    #[validate(length(min = 1))]
    pub description: String,
}

#[async_trait]
impl Entity for Todo {
    type Id = String;

    type Record<'a> = Todo;

    type Changeset = TodoChangeset;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Self::Record<'a>>, Error> {
        let todos = sqlx::query_as!(Todo, "SELECT id, description FROM todos")
            .fetch_all(executor)
            .await?;

        Ok(todos)
    }

    async fn load<'a>(
        id: String,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Todo, Error> {
        let todo = sqlx::query_as!(Todo, "SELECT id, description FROM todos WHERE id = ?", id)
            .fetch_optional(executor)
            .await?
            .ok_or(Error::NoRecordFound)?;

        Ok(todo)
    }

    async fn create<'a>(
        changeset: TodoChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Todo, Error> {
        let todo = sqlx::query_as!(
            Todo,
            "INSERT INTO todos (description) VALUES (?) RETURNING id, description",
            changeset.description
        )
        .fetch_one(executor)
        .await?;

        Ok(todo)
    }

    async fn create_batch(
        records: Vec<TodoChangeset>,
        db_pool: &SqlitePool,
    ) -> Result<Vec<Todo>, Error> {
        let mut tx = transaction(db_pool).await?;

        let mut results: Vec<Self::Record<'_>> = vec![];

        for record in records {
            let result = Self::create(record, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: String,
        record: TodoChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Todo, Error> {
        let todo = sqlx::query_as!(
            Todo,
            "UPDATE todos SET description = ? WHERE id = ? RETURNING id, description",
            record.description,
            id
        )
        .fetch_one(executor)
        .await?;

        Ok(todo)
    }

    async fn delete<'a>(
        id: String,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Todo, Error> {
        let todo = sqlx::query_as!(
            Todo,
            "DELETE FROM todos WHERE id = ? RETURNING id, description",
            id
        )
        .fetch_one(executor)
        .await?;

        Ok(todo)
    }
    async fn delete_batch(keys: Vec<Self::Id>, db_pool: &SqlitePool) -> Result<Vec<Todo>, Error> {
        let mut tx = transaction(db_pool).await?;

        let mut results: Vec<Self::Record<'_>> = vec![];

        for id in keys {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
