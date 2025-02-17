use std::{path::Path, time};

use axum_login::{AuthManagerLayerBuilder, login_required};
use nohead_rs_config::Environment;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer};
use tower_sessions::{
    ExpiredDeletion, Expiry, SessionManagerLayer,
    cookie::{Key, time::Duration},
    session_store,
};
use tower_sessions_sqlx_store::SqliteStore;
use tracing::{debug, info};

use axum::{Router, routing::get, serve};
use color_eyre::Result;
use tokio::{
    signal,
    task::{AbortHandle, JoinHandle},
};

use crate::{
    controllers::{
        Controller,
        auth::{login::LoginController, register::RegisterController},
        home::HomeController,
        todos::TodoController,
    },
    middlewares::auth::Backend,
    state::AppState,
    tracing::Tracing,
};

pub struct App {
    pub router: Router,
    pub app_state: AppState,
    pub deletion_task: JoinHandle<Result<(), session_store::Error>>,
}

impl App {
    // Builds the application without running it
    // this is useful for testing purposes
    // where axum_test will run a
    // random port
    // TODO: #1: Extract the session store setup into a separate function
    //       #2: Extract the router setup into a separate function
    pub fn build(app_state: AppState) -> Result<Self> {
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
        let backend = Backend::new(app_state.db_pool.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let static_assets = ServeDir::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("static"));

        let router = Router::new()
            .route(
                "/protected",
                get(|| async { "you gotta be logged in to see me!" }),
            )
            .route_layer(login_required!(Backend, login_url = "/auth/login"))
            .merge(HomeController::router())
            .merge(LoginController::router())
            .merge(RegisterController::router())
            .merge(TodoController::router())
            .nest_service("/static", static_assets)
            .with_state(app_state.clone())
            .layer(ServiceBuilder::new().layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(time::Duration::from_secs(10)),
                auth_layer,
            )));

        Ok(Self {
            router,
            app_state,
            deletion_task,
        })
    }

    // Serves the application on the configured
    // ip and port.
    async fn serve(app: App) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&format!(
            "{}:{}",
            app.app_state.config.server.ip, app.app_state.config.server.port
        ))
        .await?;

        debug!("listening on {}", app.app_state.config.server.addr());

        serve(listener, app.router)
            .with_graceful_shutdown(shutdown_signal(app.deletion_task.abort_handle()))
            .await?;

        App::shutdown_with_cleanup(app.deletion_task).await?;

        Ok(())
    }

    // Boots up the app on the configured binding
    // and port.
    // You can optionally hook in to
    // add graceful shutdown
    // processes.
    pub async fn boot(env: Environment) -> Result<()> {
        color_eyre::install()?;

        let app_state = AppState::build(env).await?;

        Tracing::init(&app_state.config.tracing);

        let app = App::build(app_state)?;

        App::serve(app).await?;

        Ok(())
    }

    async fn shutdown_with_cleanup(
        deletion_task: JoinHandle<Result<(), session_store::Error>>,
    ) -> Result<()> {
        match deletion_task.await {
            Ok(_) => (), // nothing to cleanup
            Err(err) if err.is_cancelled() => {
                tracing::debug!("session deletion tasks cleaned up.")
            }
            Err(err) => panic!("session deletion task failed to cleanup: {:?}", err),
        }

        info!("server shutdown successfully");

        Ok(())
    }
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
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
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
