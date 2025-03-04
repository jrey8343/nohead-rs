use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::auth::register::RegisterView,
};
use axum::{Extension, Form, Router, extract::State, response::Redirect, routing::get};
use nohead_rs_db::{
    entities::{
        register_token::RegisterToken,
        user::{RegisterUser, User},
    },
    transaction,
};
use nohead_rs_mailer::{EmailPayload, auth::AuthMailer};
use nohead_rs_worker::{Storage, WorkerStorage};

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
        Extension(mut jobs): Extension<WorkerStorage<EmailPayload>>,
        Form(form): Form<RegisterUser>,
    ) -> Result<(Flash, Redirect), Error> {
        let mut tx = transaction(&app_state.db_pool).await?;
        let user = User::create(form, &mut *tx).await?;
        let register_token = RegisterToken::create(user.id, &mut *tx).await?;
        tx.commit()
            .await
            .map_err(|e| Error::Database(nohead_rs_db::Error::DatabaseError(e)))?;

        // Send the confirmation email in a background job
        jobs.push(AuthMailer::send_confirmation(
            &app_state.email_client,
            &app_state.config,
            &user.email,
            &register_token.register_token,
        ))
        .await
        .map_err(|e| {
            tracing::error!("failed to send confirmation email: {:?}", e);
        })
        .ok();

        // Redirect to the confirmation page
        Ok((
            flash.info("please check your email for the confirmation code"),
            Redirect::to("/auth/register/confirm"),
        ))
    }
}
