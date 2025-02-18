use std::{path::Path, time::Duration};

use axum::{Router, routing::get};
use axum_login::{AuthManagerLayer, login_required};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer};
use tower_sessions_sqlx_store::SqliteStore;

use crate::{
    controllers::{
        Controller,
        auth::{login::LoginController, register::RegisterController},
        home::HomeController,
        todos::TodoController,
    },
    middlewares::auth::AuthBackend,
    state::AppState,
};

pub fn init_router(
    app_state: &AppState,
    auth_layer: AuthManagerLayer<AuthBackend, SqliteStore, tower_sessions::service::SignedCookie>,
) -> Router {
    let static_assets = ServeDir::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("static"));

    Router::new()
        .route(
            "/protected",
            get(|| async { "you gotta be logged in to see me!" }),
        )
        .route_layer(login_required!(AuthBackend, login_url = "/auth/login"))
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
            TimeoutLayer::new(Duration::from_secs(10)),
            auth_layer, // TODO: Add auth_layer
        )))
}
