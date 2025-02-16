use nohead_rs_db::{
    DbPool,
    entities::{session::Session, user::LoggedInUser},
};
use rand::RngCore;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use color_eyre::{
    Result,
    eyre::{self, OptionExt, WrapErr},
};
use thiserror::Error;

use crate::{error::Error, state::AppState};

pub const SESSION_COOKIE_NAME: &str = "nohead-sesh";

#[derive(Clone)]
pub struct AuthState(Option<(Vec<u8>, Option<LoggedInUser>, DbPool)>);

pub async fn authenticate(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Error> {
    let session_token = CookieJar::from_headers(req.headers())
        .get(SESSION_COOKIE_NAME)
        .ok_or_else(|| Error::Unauthenticated)?
        .value()
        .to_owned()
        .parse::<u8>()
        .wrap_err("error parsing session token from cookie")?;

    if let Some(session) = Session::get_by_token(session_token, &app_state.db_pool).await? {
        req.extensions_mut().insert(AuthState(Some((
            session.session_token,
            None,
            app_state.db_pool,
        ))));
    }

    Ok(next.run(req).await)
}

impl AuthState {
    pub async fn get_user(&mut self) -> Result<Option<&LoggedInUser>, Error> {
        let (session_token, store, pool) = self.0.as_mut().ok_or_else(|| Error::Unauthenticated)?;

        if store.is_none() {
            // Start transaction

            // Get the user id and email by email

            // Get the sessio n user_id
            let user: Option<(i32, String)> = sqlx::query_as(
            "SELECT id, username FROM users JOIN sessions ON user_id = id WHERE session_token = $1;"
            )
            .bind(&*session_token)
            .fetch_optional(&*pool)
            .await
                .map_err(|_| Error::Unauthenticated)?
            ;

            if let Some((_id, email)) = user {
                *store = Some(LoggedInUser { email });
            }
        }
        Ok(store.as_ref())
    }
}

fn generate_token(app_state: &AppState) -> u8 {
    let mut u128_pool = [0u8; 1];
    app_state.rng.lock().unwrap().fill_bytes(&mut u128_pool);

    u8::from_le_bytes(u128_pool)
}
