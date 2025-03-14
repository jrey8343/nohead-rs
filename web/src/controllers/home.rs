use axum::{Router, response::Response, routing::get};

use crate::{
    error::Result,
    initializers::view_engine::engine::{View, ViewEngine},
    middlewares::flash::IncomingFlashes,
    state::AppState,
    views::{self},
};

pub struct HomeController;

impl HomeController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/", get(HomeController::index))
    }
    pub async fn index(
        ViewEngine(v): ViewEngine<View>,
        flashes: IncomingFlashes,
    ) -> Result<(IncomingFlashes, Response)> {
        Ok((flashes.clone(), views::home::index(&v, flashes)))
    }
}
