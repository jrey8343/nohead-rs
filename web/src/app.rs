use std::path::Path;

use nohead_rs_config::{Config, Environment, load_config};
use tracing::debug;

use axum::{Router, routing::get, serve};
use color_eyre::Result;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::tracing::Tracing;

pub struct App {
    pub router: Router,
    pub context: Context,
}

impl App {
    // Builds the application without running it
    // this is useful for testing purposes
    // where axum_test will run a
    // random port
    pub async fn build(ctx: Context) -> Result<Self> {
        let assets = ServeDir::new(Path::new(&ctx.config.static_assets.path))
            .not_found_service(ServeFile::new("assets/404.html"));

        // build our application router and apply middlwares
        let router: Router = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .fallback_service(assets)
            .with_state(ctx.clone())
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

        Ok(Self {
            router,
            context: ctx,
        })
    }

    // Boots up the app on the configured binding
    // and port.
    // You can optionally hook in to
    // add graceful shutdown
    // processes.
    pub async fn boot(env: Environment) -> Result<()> {
        color_eyre::install()?;

        let ctx = Context::build(env).await?;

        Tracing::init(&ctx.config.tracing);

        let app = App::build(ctx).await?;

        // run it
        let listener = tokio::net::TcpListener::bind(&format!(
            "{}:{}",
            app.context.config.server.ip, app.context.config.server.port
        ))
        .await?;

        debug!("listening on {}", app.context.config.server.addr());

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

#[derive(Clone)]
pub struct Context {
    pub env: Environment,
    pub config: Config,
}

impl Context {
    pub async fn build(env: Environment) -> Result<Self> {
        let config = load_config(&env)?;
        Ok(Self { env, config })
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
