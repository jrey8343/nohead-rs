use color_eyre::eyre::{Context as _, Result};
use nohead_rs_config::get_env;
use nohead_rs_web::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let env = get_env().wrap_err("cannot get environment")?;

    App::boot(env).await.wrap_err("could not boot app")?;

    Ok(())
}
