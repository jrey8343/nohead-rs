use axum::extract::FromRef;
use axum_flash::{Config as FlashKey, Key};
use color_eyre::Result;
use nohead_rs_config::{Config, Environment, load_config};
use nohead_rs_db::{DbPool, connect_pool};

/// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
#[derive(Clone)]
pub struct AppState {
    pub env: Environment,
    pub config: Config,
    pub db_pool: DbPool,
    pub flash_key: FlashKey,
}

// Allow direct extraction of flash messages from handlers
// e.g. handler(flashes: IncomingFlashes) -> IncomingFlashes for reading
// e.g. handler(flash: Flash) -> (Flash, Redirect) for writing
impl FromRef<AppState> for FlashKey {
    fn from_ref(app_state: &AppState) -> FlashKey {
        app_state.flash_key.clone()
    }
}

impl AppState {
    pub async fn build(env: Environment) -> Result<Self> {
        let config: Config = load_config(&env)?;
        let db_pool = connect_pool(&config.database).await?;
        let flash_key = FlashKey::new(Key::generate());

        Ok(Self {
            env,
            config,
            db_pool,
            flash_key,
        })
    }
}
