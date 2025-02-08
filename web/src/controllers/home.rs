use axum::{Router, routing::get};

use crate::{state::AppState, views::home::HomeView};

pub struct HomeController;

impl HomeController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/", get(HomeController::index))
    }
    pub async fn index() -> HomeView {
        HomeView::Index
    }
}
