use axum::response::{IntoResponse, Response};
use axum_flash::IncomingFlashes;
use nohead_rs_db::entities::todo::Todo;
use rinja_axum::Template;

pub enum TodoView {
    Index(Vec<Todo>, IncomingFlashes),
    Show(Todo, IncomingFlashes),
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
            TodoView::Index(todos, _flashes) => Index { todos }.into_response(),
            TodoView::Show(todo, _flashes) => Show { todo }.into_response(),
        }
    }
}
