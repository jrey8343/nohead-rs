use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::{
    format,
    initializers::view_engine::engine::{View, ViewEngine},
    middlewares::flash::IncomingFlashes,
};

pub enum HomeView {
    Index(ViewEngine<View>, IncomingFlashes),
}

impl IntoResponse for HomeView {
    fn into_response(self) -> Response {
        match self {
            HomeView::Index(ViewEngine(v), IncomingFlashes { flashes, .. }) => format::render()
                .view(&v, "index.html", json!({ "flashes": flashes }))
                .into_response(),
        }
    }
}
