use std::path::Path;

use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_status::SetStatus,
    trace::TraceLayer,
};

use crate::state::State;

pub fn init_router(state: &State) -> axum::Router {
    axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .fallback_service(serve_assets(state))
        .with_state(state.clone())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

fn serve_assets(state: &State) -> ServeDir<SetStatus<ServeFile>> {
    ServeDir::new(Path::new(&state.config.static_assets.path))
        .not_found_service(ServeFile::new("assets/404.html"))
}
