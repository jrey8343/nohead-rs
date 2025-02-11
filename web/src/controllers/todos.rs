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

use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::todos::TodoView,
};

use super::Controller;

pub struct TodoController;

#[async_trait]
impl Controller for TodoController {
    type Id = i64;

    type View = TodoView;

    type EntityChangeset = TodoChangeset;

    type Error = Error;

    fn router() -> Router<AppState> {
        Router::new()
            .route("/todos", get(Self::read_all).post(Self::create))
            .route("/todos/batch", post(Self::create_batch))
            .route(
                "/todos/{id}",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let todos = Todo::load_all(&app_state.db_pool).await?;

        Ok((flashes.clone(), TodoView::Index(todos, flashes)))
    }

    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let todo = Todo::create(record, &app_state.db_pool).await?;

        Ok((
            flash.success("✅ created new todo"),
            Redirect::to(&format!("/todos/{}", todo.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = Todo::create_batch(records, &app_state.db_pool).await?;

        Ok((flash.success("✅ created todos"), Redirect::to("/todos")))
    }

    async fn read_one(
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let todo = Todo::load(id, &app_state.db_pool).await?;

        Ok((flashes.clone(), TodoView::Show(todo, flashes)))
    }

    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let todo = Todo::update(id, form, &app_state.db_pool).await?;

        Ok((
            flash.success("✅ updated todo"),
            Redirect::to(&format!("/todos/{}", todo.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _todo = Todo::delete(id, &app_state.db_pool).await?;

        Ok((flash.info("deleted todo"), Redirect::to("/todos")))
    }
}
