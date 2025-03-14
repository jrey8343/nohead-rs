use axum::{Router, routing::get};

use crate::{
    error::Result,
    initializers::view_engine::engine::{View, ViewEngine},
    middlewares::flash::IncomingFlashes,
    state::AppState,
    views::home::HomeView,
};

pub struct HomeController;

impl HomeController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/", get(HomeController::index))
    }

    pub async fn index(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
    ) -> Result<(IncomingFlashes, HomeView)> {
        Ok((flashes.clone(), HomeView::Index(v, flashes)))
    }
}
