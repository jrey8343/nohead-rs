use color_eyre::Result;
use nohead_rs_config::{Config, Environment, load_config};

#[derive(Clone)]
pub struct State {
    pub env: Environment,
    pub config: Config,
}

impl State {
    pub async fn build(env: Environment) -> Result<Self> {
        let config = load_config(&env)?;
        Ok(Self { env, config })
    }
}
