use axum::response::{IntoResponse, Response};
use rinja::Template;

use crate::middlewares::flash::{FlashMessage, IncomingFlashes};

use crate::views::html;

pub enum RegisterView {
    Index(IncomingFlashes),
}

#[derive(Debug, Template)]
#[template(path = "auth/register/index.html")]
pub struct Index {
    pub flashes: Vec<FlashMessage>,
}

impl IntoResponse for RegisterView {
    fn into_response(self) -> Response {
        match self {
            RegisterView::Index(flashes) => html(Index {
                flashes: flashes.flashes,
            }),
        }
    }
}
