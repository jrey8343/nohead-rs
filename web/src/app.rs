use nohead_rs_config::Environment;
use tracing::debug;

use axum::{Router, serve};
use color_eyre::Result;
use tokio::signal;

use crate::{router::init_router, state::State, tracing::Tracing};

pub struct App {
    pub router: Router,
    pub state: State,
}

impl App {
    // Builds the application without running it
    // this is useful for testing purposes
    // where axum_test will run a
    // random port
    pub async fn build(state: State) -> Result<Self> {
        let router = init_router(&state);

        Ok(Self { router, state })
    }

    // Boots up the app on the configured binding
    // and port.
    // You can optionally hook in to
    // add graceful shutdown
    // processes.
    pub async fn boot(env: Environment) -> Result<()> {
        color_eyre::install()?;

        let state = State::build(env).await?;

        Tracing::init(&state.config.tracing);

        let app = App::build(state).await?;

        // run it
        let listener = tokio::net::TcpListener::bind(&format!(
            "{}:{}",
            app.state.config.server.ip, app.state.config.server.port
        ))
        .await?;

        debug!("listening on {}", app.state.config.server.addr());

        serve(listener, app.router)
            .with_graceful_shutdown(async move {
                Self::shutdown().await;
            })
            .await?;

        Ok(())
    }

    // Tasks that need to be run on shutdown can be
    // gracefully added here.
    async fn shutdown() {
        // INFO: Add shutdown processes here
        shutdown_signal().await;
        tracing::info!("shutting down...");
    }
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
