use axum::{
    Form, Router, async_trait,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use axum_flash::{Flash, IncomingFlashes};
use nohead_rs_db::entities::{
    Entity as _,
    todo::{Todo, TodoChangeset},
};

use crate::{error::Error, state::AppState, views::todos::TodoView};

use super::Controller;

pub struct TodoController;

#[async_trait]
impl Controller for TodoController {
    type Id = String;

    type View = TodoView;

    type EntityChangeset = TodoChangeset;

    type Error = Error;

    fn router() -> axum::Router<AppState> {
        Router::new()
            .route("/", get(Self::read_all).post(Self::create))
            .route("/batch", post(Self::create_batch))
            .route(
                "/:id",
                get(Self::read_one).patch(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        State(app_state): State<AppState>,
        flashes: IncomingFlashes,
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
            flash.success("✅ successfully created todo"),
            Redirect::to(&format!("/todos/{}", todo.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = Todo::create_batch(records, &app_state.db_pool).await?;

        Ok((
            flash.success("✅ successfully created batch of todos"),
            Redirect::to("/todos"),
        ))
    }

    async fn read_one(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        flashes: IncomingFlashes,
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
            flash.success("✅ successfully updated todo"),
            Redirect::to(&format!("/todos/{}", todo.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _todo = Todo::delete(id, &app_state.db_pool).await?;

        Ok((flash.info("💡todo deleted"), Redirect::to("/todos")))
    }
}
