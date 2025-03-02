use axum::extract::State;
use axum::response::Redirect;
use axum::routing::get;
use axum::{Form, Router};
use nohead_rs_db::entities::register_token::RegisterToken;
use nohead_rs_db::entities::user::{User, UserStatus};
use nohead_rs_db::transaction;
use serde::Deserialize;

use crate::error::Error;
use crate::middlewares::auth::AuthSession;
use crate::middlewares::flash::{Flash, IncomingFlashes};
use crate::state::AppState;
use crate::views::auth::register_confirm::RegisterConfirmView;

pub struct RegisterConfirmController;

#[derive(Deserialize)]
pub struct Verify {
    register_token: String,
}

impl RegisterConfirmController {
    pub fn router() -> Router<AppState> {
        Router::new().route(
            "/auth/register/confirm",
            get(RegisterConfirmController::index).post(RegisterConfirmController::verify),
        )
    }

    pub async fn index(flashes: IncomingFlashes) -> RegisterConfirmView {
        RegisterConfirmView::Index(flashes)
    }

    pub async fn verify(
        flash: Flash,
        State(state): State<AppState>,
        mut auth_session: AuthSession,
        Form(form): Form<Verify>,
    ) -> Result<(Flash, Redirect), Error> {
        let mut tx = transaction(&state.db_pool).await?;
        // Get the user id by the user input register token
        let user_id =
            RegisterToken::try_get_user_id_by_register_token(&form.register_token, &mut *tx)
                .await?
                .ok_or_else(|| Error::InvalidRegisterToken)?;
        // Update the user status to confirmed
        let user = User::update_status(user_id, UserStatus::Confirmed, &mut *tx).await?;
        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| Error::Database(nohead_rs_db::Error::DatabaseError(e)))?;

        // Create a session for the user
        auth_session
            .login(&user)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok((
            flash.success("Welcome! You are now registered"),
            Redirect::to("/"),
        ))
    }
}
