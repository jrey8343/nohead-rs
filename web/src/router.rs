use std::path::Path;

use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_status::SetStatus,
    trace::TraceLayer,
};

use crate::state::AppState;

pub fn init_router(app_state: &AppState) -> axum::Router {
    axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .fallback_service(serve_assets(app_state))
        .with_state(app_state.clone())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

fn serve_assets(app_state: &AppState) -> ServeDir<SetStatus<ServeFile>> {
    ServeDir::new(Path::new(&app_state.config.static_assets.path))
        .not_found_service(ServeFile::new("assets/404.html"))
}
