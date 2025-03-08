use axum::response::Response;
use serde_json::json;

use crate::{error::Result, format, initializers::view_engine::engine::ViewRenderer};

pub fn index(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "test.html", json!({}))
}
