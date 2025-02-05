use axum::{Router, routing::get};
use axum_flash::IncomingFlashes;

use crate::{state::AppState, views::home::HomeView};

pub struct HomeController;

impl HomeController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/", get(HomeController::index))
    }
    pub async fn index(flashes: IncomingFlashes) -> (IncomingFlashes, HomeView) {
        (flashes.clone(), HomeView::Index(flashes))
    }
}
