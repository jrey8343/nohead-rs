use std::{path::Path, sync::Arc};

use axum::Router;
use tower_http::services::ServeDir;

use crate::{
    controllers::{Controller, home::HomeController, todos::TodoController},
    state::AppState,
};

pub fn init_router(app_state: &AppState) -> Router {
    let static_assets = ServeDir::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("static"));

    Router::new()
        .merge(HomeController::router())
        .merge(TodoController::router())
        .nest_service("/static", static_assets)
        .with_state(app_state.clone())
}
