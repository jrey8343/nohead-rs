use std::{path::Path, time::Duration};

use axum::{Extension, Router, routing::get};
use axum_login::{AuthManagerLayer, login_required};
use nohead_rs_config::Environment;
use nohead_rs_db::DeserializeOwned;
use nohead_rs_worker::WorkerStorage;
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer};
use tower_livereload::LiveReloadLayer;
use tower_sessions_sqlx_store::SqliteStore;

use crate::{
    controllers::{
        Controller,
        auth::{
            login::LoginController, logout::LogoutController, register::RegisterController,
            register_confirm::RegisterConfirmController,
        },
        home::HomeController,
        ping::PingController,
        todos::TodoController,
    },
    middlewares::auth::AuthBackend,
    state::AppState,
};

pub fn init_router<T>(
    app_state: &AppState,
    auth_layer: AuthManagerLayer<AuthBackend, SqliteStore, tower_sessions::service::SignedCookie>,
    worker_layer: WorkerStorage<T>,
) -> Router
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Unpin,
{
    let static_assets = ServeDir::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("static"));

    let mut router = Router::new()
        .route(
            "/protected",
            get(|| async { "you gotta be logged in to see me!" }),
        )
        .merge(TodoController::router())
        .route_layer(login_required!(AuthBackend, login_url = "/auth/login"))
        .merge(HomeController::router())
        .merge(LoginController::router())
        .merge(LogoutController::router())
        .merge(RegisterController::router())
        .merge(RegisterConfirmController::router())
        .merge(PingController::router())
        .nest_service("/static", static_assets)
        .with_state(app_state.clone())
        .layer(ServiceBuilder::new().layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            auth_layer,
            Extension(worker_layer),
        )));

    // enable live reload in development
    if app_state.env == Environment::Development {
        router = router.layer(LiveReloadLayer::new());
    }

    router
}
