use axum::{Form, Router, extract::State, response::Redirect, routing::get};
use nohead_rs_db::{
    entities::{
        register_token::{RegisterToken, RegisterTokenValidate},
        user::{User, UserStatus},
    },
    transaction,
};

use crate::{
    error::Error,
    initializers::view_engine::engine::{View, ViewEngine},
    middlewares::{
        auth::AuthSession,
        flash::{Flash, IncomingFlashes},
    },
    state::AppState,
    views::auth::register_confirm::RegisterConfirmView,
};

pub struct RegisterConfirmController;

impl RegisterConfirmController {
    pub fn router() -> Router<AppState> {
        Router::new().route(
            "/auth/register/confirm",
            get(RegisterConfirmController::index).post(RegisterConfirmController::verify),
        )
    }

    pub async fn index(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
    ) -> (IncomingFlashes, RegisterConfirmView) {
        (flashes.clone(), RegisterConfirmView::Index(v, flashes))
    }

    pub async fn verify(
        flash: Flash,
        State(state): State<AppState>,
        mut auth_session: AuthSession,
        Form(form): Form<RegisterTokenValidate>,
    ) -> Result<(Flash, Redirect), Error> {
        let mut tx = transaction(&state.db_pool).await?;
        // Get the user id by the user input register token
        let user_id = RegisterToken::try_get_user_id_by_register_token(form, &mut *tx)
            .await?
            .ok_or_else(|| Error::InvalidRegisterToken)?;
        // Update the user status to from pending to confirmed
        let user = User::update_status(user_id, UserStatus::Confirmed, &mut *tx).await?;
        // Commit the transaction
        tx.commit().await.map_err(|e| Error::Database(e.into()))?;

        // Create a session for the user
        auth_session
            .login(&user)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        //FIX: Create a better error to allow user
        //to retry

        Ok((
            flash.success("Welcome! You are now registered"),
            Redirect::to("/"),
        ))
    }
}
