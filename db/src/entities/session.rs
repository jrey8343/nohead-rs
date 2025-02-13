use serde::Deserialize;
use sqlx::Sqlite;
use sqlx::prelude::FromRow;

#[derive(Clone, FromRow, Deserialize, Debug)]
pub struct Session {
    pub session_token: Vec<u8>,
    pub user_id: i64,
}

impl Session {
    /// ------------------------------------------------------------------------
    /// Create a new session in the database
    /// ------------------------------------------------------------------------
    ///
    /// # Parameters
    ///
    /// - `session`  - The session object to insert into the database.
    /// - `executor` - The database executor.
    ///
    /// # Returns
    /// - A session object with the session token and user ID.
    ///
    /// ------------------------------------------------------------------------
    pub async fn create(
        session: Session,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Session, sqlx::Error> {
        let session = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions (session_token, user_id)
            VALUES (?, ?)
            RETURNING *

"#,
            session.session_token,
            session.user_id
        )
        .fetch_one(executor)
        .await?;

        Ok(session)
    }
}
