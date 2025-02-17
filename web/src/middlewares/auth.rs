use async_trait::async_trait;
use axum_login::{AuthManagerLayerBuilder, AuthnBackend, UserId};
use nohead_rs_db::{
    DbPool,
    entities::user::{User, UserCredentials},
};
use password_auth::verify_password;
use tokio::task::{self, JoinHandle};
use tower_sessions::{
    ExpiredDeletion, Expiry, SessionManagerLayer,
    cookie::{Key, time::Duration},
    session_store,
};
use tower_sessions_sqlx_store::SqliteStore;

use crate::{error::Error, state::AppState};

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Debug, Clone)]
pub struct Backend {
    db: DbPool,
}

impl Backend {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

/// ------------------------------------------------------------------------
/// Specific authentication related queries for the User entity.
/// ------------------------------------------------------------------------
#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = UserCredentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = User::try_get_by_email(&creds.email, &self.db).await?;
        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        task::spawn_blocking(|| {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(creds.password, &user.password_hash).is_ok()))
        })
        .await
        .map_err(|e| Error::Unexpected(e.into()))?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = User::try_get_by_id(user_id, &self.db).await?;
        Ok(user)
    }
}
