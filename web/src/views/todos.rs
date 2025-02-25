use axum::response::{IntoResponse, Response};
use nohead_rs_db::entities::todo::Todo;
use rinja::Template;

use crate::middlewares::flash::{FlashMessage, IncomingFlashes};

use super::html;

pub enum TodoView {
    Index(Vec<Todo>, IncomingFlashes),
    Show(Todo, IncomingFlashes),
}

#[derive(Debug, Template)]
#[template(path = "todos/index.html")]
pub struct Index {
    pub todos: Vec<Todo>,
    pub flashes: Vec<FlashMessage>,
}

#[derive(Debug, Template)]
#[template(path = "todos/show.html")]
pub struct Show {
    pub todo: Todo,
    pub flashes: Vec<FlashMessage>,
}

impl IntoResponse for TodoView {
    fn into_response(self) -> Response {
        match self {
            TodoView::Index(todos, IncomingFlashes { flashes, .. }) => {
                html(Index { todos, flashes })
            }
            TodoView::Show(todo, IncomingFlashes { flashes, .. }) => html(Show { todo, flashes }),
        }
    }
}
