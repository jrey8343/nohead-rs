use axum::response::{IntoResponse, Response};
use rinja::Template;

use crate::middlewares::flash::{FlashMessage, IncomingFlashes};

use super::html;

pub enum HomeView {
    Index(IncomingFlashes),
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct Index {
    flashes: Vec<FlashMessage>,
}

impl IntoResponse for HomeView {
    fn into_response(self) -> Response {
        match self {
            HomeView::Index(IncomingFlashes { flashes, .. }) => html(Index { flashes }),
        }
    }
}
