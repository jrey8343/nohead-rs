use nohead_rs_config::Environment;
use nohead_rs_worker::Worker;
use tower_sessions::session_store;
use tracing::{debug, info};

use axum::{Router, serve};
use color_eyre::Result;
use tokio::{
    net::TcpListener,
    signal,
    task::{AbortHandle, JoinHandle},
};

use crate::{
    initializers::view_engine::engine::ViewEngineInitializer,
    middlewares::auth::AuthSessionManager, router::init_router, state::AppState, tracing::Tracing,
};

pub struct App {
    pub router: Router,
    pub app_state: AppState,
    pub deletion_task: JoinHandle<Result<(), session_store::Error>>,
    pub worker_monitor_task: JoinHandle<Result<(), std::io::Error>>,
}

impl App {
    // Builds the application without running it
    // this is useful for testing purposes
    // where axum_test will run a
    // random port
    pub fn build(app_state: AppState) -> Result<Self> {
        let AuthSessionManager {
            deletion_task,
            auth_layer,
        } = AuthSessionManager::new(&app_state);

        // Initialize the background worker
        let worker = Worker::new(&app_state.db_pool, app_state.email_client.clone());

        // Initialize the view engine
        let view_engine = ViewEngineInitializer::default();
        view_engine.before_run(app_state.clone())?;

        // Initialize the router
        let router = init_router(&app_state, auth_layer, worker.storage, view_engine)?;

        Ok(Self {
            router,
            app_state,
            deletion_task,
            worker_monitor_task: worker.monitor_task,
        })
    }

    // Serves the application on the configured
    // ip and port.
    async fn serve(app: App) -> Result<()> {
        let listener = TcpListener::bind(&app.app_state.config.server.addr()).await?;

        debug!(
            "listening on {}:{}",
            app.app_state.config.server.host, app.app_state.config.server.port
        );

        serve(listener, app.router)
            .with_graceful_shutdown(shutdown_signal(vec![
                app.deletion_task.abort_handle(),
                app.worker_monitor_task.abort_handle(),
            ]))
            .await?;

        App::shutdown_with_cleanup(app.deletion_task, app.worker_monitor_task).await?;

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
        monitor_task: JoinHandle<Result<(), std::io::Error>>,
    ) -> Result<()> {
        match deletion_task.await {
            Ok(_) => (), // nothing to cleanup
            Err(err) if err.is_cancelled() => {
                tracing::debug!("session deletion tasks cleaned up.")
            }
            Err(err) => panic!("session deletion task failed to cleanup: {:?}", err),
        }

        match monitor_task.await {
            Ok(_) => (), // nothing to cleanup
            Err(err) if err.is_cancelled() => {
                tracing::debug!("worker monitor task cleaned up.")
            }
            Err(err) => panic!("worker monitor task failed to cleanup: {:?}", err),
        }

        info!("server shutdown successfully");

        Ok(())
    }
}

async fn shutdown_signal(task_handles: Vec<AbortHandle>) {
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
        _ = ctrl_c => { for task_handle in task_handles {
            task_handle.abort();
        } },
        _ = terminate => { for task_handle in task_handles {
            task_handle.abort();
        } },
    }
}
