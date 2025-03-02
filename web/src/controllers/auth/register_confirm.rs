use axum::Router;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
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
pub struct Params {
    register_token: Option<String>,
}

impl RegisterConfirmController {
    pub fn router() -> Router<AppState> {
        Router::new().route(
            "/auth/register/confirm",
            get(RegisterConfirmController::index),
        )
    }

    pub async fn index(
        mut auth_session: AuthSession,
        flash: Flash,
        flashes: IncomingFlashes,
        State(state): State<AppState>,
        Query(params): Query<Params>,
    ) -> Result<(Flash, impl IntoResponse), Error> {
        // If there is no register token in the params then just return the view
        if params.register_token.is_none() {
            return Ok((
                flash.info("Check your email for the confirmation link"),
                RegisterConfirmView::Index(flashes).into_response(),
            ));
        }

        let mut tx = transaction(&state.db_pool).await?;
        // Get the user id by the register token
        let user_id = RegisterToken::try_get_user_id_by_register_token(
            &params.register_token.unwrap(),
            &mut *tx,
        )
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
            Redirect::to("/").into_response(),
        ))
    }
}
