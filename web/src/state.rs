use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use color_eyre::Result;
use nohead_rs_config::{Config, Environment, load_config};
use nohead_rs_db::{DbPool, connect_pool};

use crate::middlewares::flash;

/// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
#[derive(Clone)]
pub struct AppState {
    pub env: Environment,
    pub config: Config,
    pub db_pool: DbPool,
    pub flash_config: flash::Config,
}

impl AppState {
    pub async fn build(env: Environment) -> Result<Self> {
        let config: Config = load_config(&env)?;
        let db_pool = connect_pool(&config.database).await?;
        let flash_config = flash::Config::new(Key::generate());

        Ok(Self {
            env,
            config,
            db_pool,
            flash_config,
        })
    }
}

/// Allow direct extraction of flash messages in handlers.
impl FromRef<AppState> for flash::Config {
    fn from_ref(app_state: &AppState) -> flash::Config {
        app_state.flash_config.clone()
    }
}
