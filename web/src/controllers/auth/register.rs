use axum::Router;
use axum::extract::State;
use axum::routing::get;
use axum::{Form, response::Redirect};
use nohead_rs_db::entities::register_token::RegisterToken;
use nohead_rs_db::entities::user::{RegisterUser, User};
use nohead_rs_db::transaction;
use nohead_rs_mailer::emails::send_register_confirm_email;

use crate::error::Error;
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
        flash: Flash,
        State(app_state): State<AppState>,
        Form(form): Form<RegisterUser>,
    ) -> Result<(Flash, Redirect), Error> {
        let mut tx = transaction(&app_state.db_pool).await?;
        let user = User::create(form, &mut *tx).await?;
        let register_token = RegisterToken::create(user.id, &mut *tx).await?;
        tx.commit()
            .await
            .map_err(|e| Error::Database(nohead_rs_db::Error::DatabaseError(e)))?;

        // Send the confirmation email
        send_register_confirm_email(
            &app_state.email_client,
            &user.email,
            &register_token.register_token,
        )
        .await?;

        // Redirect to the confirmation page
        Ok((
            flash.info("please check your email for the confirmation link"),
            Redirect::to("/auth/register/confirm"),
        ))
    }
}
