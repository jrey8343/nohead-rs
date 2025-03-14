use axum::response::{IntoResponse, Response};
use rinja::Template;
use serde_json::json;

use crate::{
    error::Result,
    format,
    initializers::view_engine::engine::ViewRenderer,
    middlewares::flash::{FlashMessage, IncomingFlashes},
};

use super::html;

// pub enum HomeView {
//     Index(IncomingFlashes),
// }
//
// #[derive(Template, Debug)]
// #[template(path = "index.html")]
// pub struct Index {
//     flashes: Vec<FlashMessage>,
// }
//
// impl IntoResponse for HomeView {
//     fn into_response(self) -> Response {
//         match self {
//             HomeView::Index(IncomingFlashes { flashes, .. }) => html(Index { flashes }),
//         }
//     }
// }

pub fn index(
    v: &impl ViewRenderer,
    IncomingFlashes { flashes, .. }: IncomingFlashes,
) -> Result<Response> {
    format::render().view(v, "index.html", json!({ "flashes": flashes }))
}
