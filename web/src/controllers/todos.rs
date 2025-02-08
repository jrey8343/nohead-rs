use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use nohead_rs_db::entities::{
    Entity as _,
    todo::{Todo, TodoChangeset},
};

use crate::{error::Error, state::AppState, views::todos::TodoView};

use super::Controller;

pub struct TodoController;

#[async_trait]
impl Controller for TodoController {
    type Id = i64;

    type View = TodoView;

    type EntityChangeset = TodoChangeset;

    type Error = Error;

    fn router() -> axum::Router<AppState> {
        Router::new()
            .route("/", get(Self::read_all).post(Self::create))
            .route("/batch", post(Self::create_batch))
            .route(
                "/:id",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(State(app_state): State<AppState>) -> Result<Self::View, Self::Error> {
        let todos = Todo::load_all(&app_state.db_pool).await?;

        Ok(TodoView::Index(todos))
    }

    async fn create(
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<Redirect, Self::Error> {
        let todo = Todo::create(record, &app_state.db_pool).await?;

        Ok(Redirect::to(&format!("/todos/{}", todo.id)))
    }

    async fn create_batch(
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<Redirect, Self::Error> {
        let _records = Todo::create_batch(records, &app_state.db_pool).await?;

        Ok(Redirect::to("/todos"))
    }

    async fn read_one(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<Self::View, Self::Error> {
        let todo = Todo::load(id, &app_state.db_pool).await?;

        Ok(TodoView::Show(todo))
    }

    async fn update(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<Redirect, Self::Error> {
        let todo = Todo::update(id, form, &app_state.db_pool).await?;

        Ok(Redirect::to(&format!("/todos/{}", todo.id)))
    }

    async fn delete(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<Redirect, Self::Error> {
        let _todo = Todo::delete(id, &app_state.db_pool).await?;

        Ok(Redirect::to("/todos"))
    }
}
