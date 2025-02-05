use nohead_rs_config::DatabaseConfig;
use sqlx::{Sqlite, Transaction, sqlite::SqlitePoolOptions};

pub use sqlx::SqlitePool as DbPool;

/// Entity definitions and related functions
pub mod entities;

/// Starts a new database transaction.
///
/// Example:
/// ```
/// let tx = transaction(&app_state.db_pool).await?;
/// tasks::create(task_data, &mut *tx)?;
/// users::create(user_data, &mut *tx)?;
///
/// match tx.commit().await {
///     Ok(_) => Ok((StatusCode::CREATED, TasksView(results))),
///     Err(e) => Err((internal_error(e), "".into())),
/// }
/// ```
///
/// Transactions are rolled back automatically when they are dropped without having been committed.
pub async fn transaction(db_pool: &DbPool) -> Result<Transaction<'static, Sqlite>, Error> {
    let tx = db_pool.begin().await?;

    Ok(tx)
}

/// Creates a connection pool to the database specified in the passed [`{{project-name}}-config::DatabaseConfig`]
pub async fn connect_pool(config: &DatabaseConfig) -> Result<DbPool, Error> {
    let pool = SqlitePoolOptions::new()
        .connect(config.url.as_str())
        .await?;

    Ok(pool)
}

/// Errors that can occur as a result of a data layer operation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// General database error, e.g. communicating with the database failed
    #[error("database query failed")]
    DbError(#[from] sqlx::Error),
    #[error("validation failed")]
    /// An invalid changeset was passed to a writing operation such as creating or updating a record.
    ValidationError(#[from] validator::ValidationErrors),
}
