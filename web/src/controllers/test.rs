use axum::{Router, response::Response, routing::get};

use crate::{
    error::Result,
    initializers::view_engine::engine::{View, ViewEngine, ViewRenderer},
    state::AppState,
    views::{self},
};

pub struct TestController;

impl TestController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/test", get(TestController::index))
    }
    pub async fn index(ViewEngine(v): ViewEngine<View>) -> Result<Response> {
        views::test::index(&v)
    }
}
