use serde::Deserialize;
use sqlx::Sqlite;
use sqlx::prelude::FromRow;

use crate::Error;

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
    ) -> Result<Session, Error> {
        let session = sqlx::query_as!(
            Session,
            r#"
            insert into sessions (session_token, user_id)
            values (?, ?)
            returning *

"#,
            session.session_token,
            session.user_id
        )
        .fetch_one(executor)
        .await?;

        Ok(session)
    }

    pub async fn get_by_token(
        session_token: u8,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Option<Session>, Error> {
        let session = sqlx::query_as!(
            Session,
            r#"
        select * from sessions where session_token = ?

"#,
            session_token
        )
        .fetch_optional(executor)
        .await
        .map_err(|_| Error::NoRecordFound)?;

        Ok(session)
    }
}
