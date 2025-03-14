use axum::response::{IntoResponse, Response};
use nohead_rs_db::entities::todo::Todo;
use serde_json::json;

use crate::{
    format,
    initializers::view_engine::engine::{View, ViewEngine},
    middlewares::flash::IncomingFlashes,
};

pub enum TodoView {
    Index(ViewEngine<View>, Vec<Todo>, IncomingFlashes),
    Show(ViewEngine<View>, Todo, IncomingFlashes),
}

impl IntoResponse for TodoView {
    fn into_response(self) -> Response {
        match self {
            TodoView::Index(ViewEngine(v), todos, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "todos/index.html",
                        json!({ "todos": todos, "flashes": flashes }),
                    )
                    .into_response()
            }
            TodoView::Show(ViewEngine(v), todo, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "todos/show.html",
                        json!({ "todo": todo, "flashes": flashes }),
                    )
                    .into_response()
            }
        }
    }
}
