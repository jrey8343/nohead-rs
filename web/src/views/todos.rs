use axum::response::{IntoResponse, Response};
use nohead_rs_db::entities::todo::Todo as TodoEntity;
use rinja_axum::Template;

pub enum TodoView {
    Index(Index),
}

#[derive(Debug, Template)]
#[template(path = "todos/index.html")]
pub struct Index {
    todos: Vec<TodoEntity>,
}

impl IntoResponse for TodoView {
    fn into_response(self) -> Response {
        match self {
            TodoView::Index(v) => v.into_response(),
        }
    }
}
