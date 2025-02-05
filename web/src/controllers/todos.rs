use axum::{
    async_trait,
    extract::{Path, State},
};
use nohead_rs_db::entities::todo::TodoChangeset;

use crate::{error::Error, state::SharedAppState, views::todos::TodoView};

use super::Controller;

pub struct TodoController;

#[async_trait]
impl Controller for TodoController {
    type Id = String;

    type View = TodoView;

    type EntityChangeset = TodoChangeset;

    type Error = Error;

    fn router() -> axum::Router<SharedAppState> {
        todo!()
    }

    async fn read_all(
        State(app_state): State<SharedAppState>,
        flashes: axum_flash::IncomingFlashes,
    ) -> Result<(axum_flash::IncomingFlashes, Self::View), Self::Error> {
        todo!()
    }

    async fn create(
        flash: axum_flash::Flash,
        State(app_state): State<SharedAppState>,
        axum::Form(record): axum::Form<Self::EntityChangeset>,
    ) -> Result<(axum_flash::Flash, axum::response::Redirect), Self::Error> {
        todo!()
    }

    async fn create_batch(
        flash: axum_flash::Flash,
        State(app_state): State<SharedAppState>,
        axum::Form(records): axum::Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(axum_flash::Flash, axum::response::Redirect), Self::Error> {
        todo!()
    }

    async fn read_one(
        Path(id): Path<Self::Id>,
        State(app_state): State<SharedAppState>,
        flashes: axum_flash::IncomingFlashes,
    ) -> Result<(axum_flash::IncomingFlashes, Self::View), Self::Error> {
        todo!()
    }

    async fn update(
        flash: axum_flash::Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<SharedAppState>,
        form: axum::Form<Self::EntityChangeset>,
    ) -> Result<(axum_flash::Flash, axum::response::Redirect), Self::Error> {
        todo!()
    }

    async fn delete(
        flash: axum_flash::Flash,
        Path(id): Path<i64>,
        State(app_state): State<SharedAppState>,
    ) -> Result<(axum_flash::Flash, axum::response::Redirect), Self::Error> {
        todo!()
    }
}
