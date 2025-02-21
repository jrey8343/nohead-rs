use axum::Router;
use axum::extract::Query;
use axum::routing::get;
use axum::{Form, response::Redirect};
use nohead_rs_db::entities::user::UserCredentials;
use serde::Deserialize;

use crate::error::Error;
use crate::middlewares::auth::AuthSession;
use crate::middlewares::flash::{Flash, IncomingFlashes};
use crate::state::AppState;
use crate::views::auth::login::LoginView;

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub struct LoginController;

impl LoginController {
    pub fn router() -> Router<AppState> {
        Router::new().route(
            "/auth/login",
            get(LoginController::index).post(LoginController::login),
        )
    }

    pub async fn index(
        Query(NextUrl { next }): Query<NextUrl>,
        flashes: IncomingFlashes,
    ) -> (IncomingFlashes, LoginView) {
        (flashes.clone(), LoginView::Index(flashes, next))
    }

    pub async fn login(
        mut auth_session: AuthSession,
        flash: Flash,
        Form(creds): Form<UserCredentials>,
    ) -> Result<(Flash, Redirect), Error> {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                let mut login_url = "/auth/login".to_string();
                if let Some(next) = creds.next {
                    login_url = format!("{}?next={}", login_url, next);
                };
                return Ok((
                    flash.error("❌ invalid credentials"),
                    Redirect::to(&login_url),
                ));
            }
            Err(e) => return Err(Error::Unexpected(e.into())),
        };

        auth_session
            .login(&user)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        if let Some(ref next) = creds.next {
            Ok((
                flash.success("✅ successfully logged in"),
                Redirect::to(next),
            ))
        } else {
            Ok((
                flash.success("✅ successfully logged in"),
                Redirect::to("/"),
            ))
        }
    }
}
