use std::path::Path;

use axum::{Router, extract::Request, routing::get};
use axum_login::login_required;
use tower::{Layer, Service, ServiceBuilder};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{
    controllers::{
        Controller,
        auth::{login::LoginController, register::RegisterController},
        home::HomeController,
        todos::TodoController,
    },
    middlewares::auth::Backend,
    state::AppState,
};

// TODO: Use newtype pattern to wrap the AppState
// then we can initialize and add other methods

pub fn init_router(app_state: &AppState) -> Router {
    let static_assets = ServeDir::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("static"));
    Router::new()
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
}

pub fn after_routes(router: Router) -> Router {
    router.layer(TraceLayer::new_for_http())
}
