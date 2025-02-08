use axum::response::{IntoResponse, Response};
use nohead_rs_db::entities::todo::Todo;
use rinja::Template;

use crate::error::Error;

pub enum TodoView {
    Index(Vec<Todo>),
    Show(Todo),
}

#[derive(Debug, Template)]
#[template(path = "todos/index.html")]
pub struct Index {
    pub todos: Vec<Todo>,
}

#[derive(Debug, Template)]
#[template(path = "todos/show.html")]
pub struct Show {
    pub todo: Todo,
}

impl IntoResponse for TodoView {
    fn into_response(self) -> Response {
        match self {
            TodoView::Index(todos) => Index { todos }
                .render()
                .map_err(Error::Render)
                .into_response(),
            TodoView::Show(todo) => Show { todo }
                .render()
                .map_err(Error::Render)
                .into_response(),
        }
    }
}
