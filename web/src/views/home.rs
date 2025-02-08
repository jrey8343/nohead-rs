use axum::response::{IntoResponse, Response};
use rinja::Template;

use super::html;

pub enum HomeView {
    Index,
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct Index {}

impl IntoResponse for HomeView {
    fn into_response(self) -> Response {
        match self {
            HomeView::Index => html(Index {}),
        }
    }
}
