use axum::response::{IntoResponse, Response};
use rinja::Template;

use crate::middlewares::flash::{FlashMessage, IncomingFlashes};

use crate::views::html;

pub enum RegisterConfirmView {
    Index(IncomingFlashes),
}

#[derive(Debug, Template)]
#[template(path = "auth/register_confirm/index.html")]
pub struct Index {
    pub flashes: Vec<FlashMessage>,
}

impl IntoResponse for RegisterConfirmView {
    fn into_response(self) -> Response {
        match self {
            RegisterConfirmView::Index(IncomingFlashes { flashes, .. }) => html(Index { flashes }),
        }
    }
}
