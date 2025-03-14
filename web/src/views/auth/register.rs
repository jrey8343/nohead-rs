use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::format;
use crate::initializers::view_engine::engine::{View, ViewEngine};
use crate::middlewares::flash::IncomingFlashes;

pub enum RegisterView {
    Index(ViewEngine<View>, IncomingFlashes),
}

impl IntoResponse for RegisterView {
    fn into_response(self) -> Response {
        match self {
            RegisterView::Index(ViewEngine(v), IncomingFlashes { flashes, .. }) => format::render()
                .view(&v, "auth/register/index.html", json!({"flashes": flashes}))
                .into_response(),
        }
    }
}
