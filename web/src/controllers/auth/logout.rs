use axum::{Router, response::Redirect, routing::post};

use crate::{
    error::Error,
    middlewares::{auth::AuthSession, flash::Flash},
    state::AppState,
};

pub struct LogoutController;

impl LogoutController {
    pub fn router() -> Router<AppState> {
        Router::new().route("/auth/logout", post(LogoutController::logout))
    }
    pub async fn logout(
        mut auth_session: AuthSession,
        flash: Flash,
    ) -> Result<(Flash, Redirect), Error> {
        auth_session
            .logout()
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok((
            flash.success("You have been logged out"),
            Redirect::to("/auth/login"),
        ))
    }
}
