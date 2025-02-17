use axum::Router;
use axum::extract::State;
use axum::routing::get;
use axum::{Form, response::Redirect};
use nohead_rs_db::entities::user::{RegisterUser, User};

use crate::error::Error;
use crate::middlewares::auth::AuthSession;
use crate::middlewares::flash::{Flash, IncomingFlashes};
use crate::state::AppState;
use crate::views::auth::register::RegisterView;

pub struct RegisterController;

impl RegisterController {
    pub fn router() -> Router<AppState> {
        Router::new().route(
            "/auth/register",
            get(RegisterController::index).post(RegisterController::register),
        )
    }

    pub async fn index(flashes: IncomingFlashes) -> (IncomingFlashes, RegisterView) {
        (flashes.clone(), RegisterView::Index(flashes))
    }

    pub async fn register(
        mut auth_session: AuthSession,
        flash: Flash,
        State(app_state): State<AppState>,
        Form(form): Form<RegisterUser>,
    ) -> Result<(Flash, Redirect), Error> {
        let user = User::create(form, &app_state.db_pool).await?;

        auth_session
            .login(&user)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok((
            flash.success("✅ successfully logged in!"),
            Redirect::to("/"),
        ))
    }
}
