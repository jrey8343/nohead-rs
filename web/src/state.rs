use std::sync::Arc;

use color_eyre::Result;
use nohead_rs_config::{Config, Environment, load_config};
use nohead_rs_db::{DbPool, connect_pool};

/// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
#[derive(Clone)]
pub struct AppState {
    pub env: Environment,
    pub config: Config,
    pub db_pool: DbPool,
}

impl AppState {
    pub async fn build(env: Environment) -> Result<Self> {
        let config: Config = load_config(&env)?;
        let db_pool = connect_pool(&config.database).await?;

        Ok(Self {
            env,
            config,
            db_pool,
        })
    }
}

/// The application's state as it is shared across the application, e.g. in controllers and middlewares.
///
/// This is the [`AppState`] struct wrappend in an [`std::sync::Arc`].
pub type SharedAppState = Arc<AppState>;
