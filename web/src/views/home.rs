use axum::response::{IntoResponse, Response};
use axum_flash::IncomingFlashes;
use rinja::Template;

pub enum HomeView {
    Index(IncomingFlashes),
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct Index {}

impl IntoResponse for HomeView {
    fn into_response(self) -> Response {
        match self {
            HomeView::Index(flashes) => Index {}.into_response(),
        }
    }
}
