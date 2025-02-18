use async_trait::async_trait;
use axum_login::{AuthManagerLayer, AuthManagerLayerBuilder, AuthnBackend, UserId};
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
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

#[derive(Debug, Clone)]
pub struct AuthBackend {
    db: DbPool,
}

impl AuthBackend {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

// ------------------------------------------------------------------------
/// Specific authentication related queries for the User entity.
/// ------------------------------------------------------------------------
#[async_trait]
impl AuthnBackend for AuthBackend {
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

/// ------------------------------------------------------------------------
/// A canvenience struct to build and manage the authentication session.
/// ------------------------------------------------------------------------
/// # Returns
///
/// A struct that contains the deletion task for cleanup
/// and the auth layer middleware for our router.
///
/// ------------------------------------------------------------------------
pub struct AuthSessionManager {
    pub deletion_task: JoinHandle<Result<(), session_store::Error>>,
    pub auth_layer:
        AuthManagerLayer<AuthBackend, SqliteStore, tower_sessions::service::SignedCookie>,
}

impl AuthSessionManager {
    pub fn new(app_state: &AppState) -> Self {
        // Connect to the session store in sqlite.
        let session_store = SqliteStore::new(app_state.db_pool.clone())
            .with_table_name("sessions")
            .expect("unable to connect to session store");
        // Panic here as this is a fatal error
        // at startup.

        let deletion_task = tokio::task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );

        // Generate a cryptographic key to sign the session cookie.
        let key = Key::generate();

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(true)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)))
            .with_signed(key);

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = AuthBackend::new(app_state.db_pool.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        Self {
            deletion_task,
            auth_layer,
        }
    }
}
