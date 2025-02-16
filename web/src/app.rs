use std::path::Path;

use axum_login::AuthManagerLayerBuilder;
use nohead_rs_config::Environment;
use nohead_rs_db::entities::user::Backend;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{
    ExpiredDeletion, Expiry, SessionManagerLayer,
    cookie::{Key, time::Duration},
    session_store,
};
use tower_sessions_sqlx_store::SqliteStore;
use tracing::debug;

use axum::{Router, serve};
use color_eyre::Result;
use tokio::{
    signal,
    task::{AbortHandle, JoinError, JoinHandle},
};

use crate::{
    controllers::{Controller, home::HomeController, todos::TodoController},
    state::AppState,
    tracing::Tracing,
};

struct App {
    pub router: Router,
    pub app_state: AppState,
    deletion_task: JoinHandle<Result<(), session_store::Error>>,
}

impl App {
    // Builds the application without running it
    // this is useful for testing purposes
    // where axum_test will run a
    // random port
    async fn build(app_state: AppState) -> Result<Self> {
        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let session_store = SqliteStore::new(app_state.db_pool.clone());

        let deletion_task = tokio::task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );

        // Generate a cryptographic key to sign the session cookie.
        let key = Key::generate();

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)))
            .with_signed(key);

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(app_state.db_pool.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        // let router = init_router(&app_state);
        let static_assets = ServeDir::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("static"));

        let router = Router::new()
            .merge(HomeController::router())
            .merge(TodoController::router())
            .nest_service("/static", static_assets)
            .with_state(app_state.clone())
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
            .layer(auth_layer);

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

        app.deletion_task.await??;

        Ok(())
    }

    // Boots up the app on the configured binding
    // and port.
    // You can optionally hook in to
    // add graceful shutdown
    // processes.
    async fn boot(env: Environment) -> Result<()> {
        color_eyre::install()?;

        let app_state = AppState::build(env).await?;

        Tracing::init(&app_state.config.tracing);

        let app = App::build(app_state).await?;

        App::serve(app).await?;

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
