use axum::response::{IntoResponse, Response};
use rinja::Template;

use crate::error::Error;

pub enum HomeView {
    Index,
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct Index {}

impl IntoResponse for HomeView {
    fn into_response(self) -> Response {
        match self {
            HomeView::Index => Index {}.render().map_err(Error::Render).into_response(),
        }
    }
}
