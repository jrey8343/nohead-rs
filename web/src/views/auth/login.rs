use axum::response::{IntoResponse, Response};
use rinja::Template;

use crate::middlewares::flash::{FlashMessage, IncomingFlashes};

use crate::views::html;

pub enum LoginView {
    Index(IncomingFlashes, Option<String>),
}

#[derive(Debug, Template)]
#[template(path = "auth/login/index.html")]
pub struct Index {
    pub flashes: Vec<FlashMessage>,
    next: Option<String>,
}

impl IntoResponse for LoginView {
    fn into_response(self) -> Response {
        match self {
            LoginView::Index(flashes, next) => html(Index {
                flashes: flashes.flashes,
                next,
            }),
        }
    }
}
