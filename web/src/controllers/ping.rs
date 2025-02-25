use axum::{Router, http::StatusCode, routing::get};

use crate::state::AppState;

pub struct PingController;

impl PingController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/ping", get(PingController::ping))
    }
    pub async fn ping() -> StatusCode {
        StatusCode::OK
    }
}
