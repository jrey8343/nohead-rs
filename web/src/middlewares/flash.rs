//! One-time notifications (aka flash messages) middleware.
//!
//! # Example
//!
//! ```
//! use axum::{
//!     response::{IntoResponse, Redirect},
//!     extract::FromRef,
//!     routing::get,
//!     Router,
//! };
//! use crate::middlewares::flash::{IncomingFlashes, Flash, Key, Config};
//!
//! #[derive(Clone)]
//! struct AppState {
//!     flash_config: Config,
//! }
//!
//! let app_state = AppState {
//!     // The key should probably come from configuration
//!     flash_config: Config::new(Key::generate()),
//! };
//!
//! // Our state type must implement this trait. That is how the config
//! // is passed to axum-flash in a type safe way.
//! impl FromRef<AppState> for Config {
//!     fn from_ref(state: &AppState) -> Config {
//!         state.flash_config.clone()
//!     }
//! }
//!
//! let app = Router::new()
//!     .route("/", get(root))
//!     .route("/set-flash", get(set_flash))
//!     .with_state(app_state);
//!
//! async fn root(flashes: IncomingFlashes) -> IncomingFlashes {
//!     for (level, text) in &flashes {
//!         // ...
//!     }
//!
//!     // The flashes must be returned so the cookie is removed
//!     flashes
//! }
//!
//! async fn set_flash(flash: Flash) -> (Flash, Redirect) {
//!     (
//!         // The flash must be returned so the cookie is set
//!         flash.debug("Hi from flash!"),
//!         Redirect::to("/"),
//!     )
//! }
//! # let _: Router = app;
//! ```
//!

use axum::http::{StatusCode, request::Parts};
use axum::{
    extract::{FromRef, FromRequestParts},
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use axum_extra::extract::cookie::{self, Cookie, SignedCookieJar};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};
use std::{
    convert::{Infallible, TryInto},
    time::Duration,
};

pub use axum_extra::extract::cookie::Key;

/// Extractor for setting outgoing flash messages.
///
/// The flashes will be stored in a signed cookie.
#[derive(Clone)]
pub struct Flash {
    flashes: Vec<FlashMessage>,
    use_secure_cookies: bool,
    key: Key,
}

impl fmt::Debug for Flash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flash")
            .field("flashes", &self.flashes)
            .field("use_secure_cookies", &self.use_secure_cookies)
            .field("key", &"REDACTED")
            .finish()
    }
}

impl Flash {
    /// Push an `Debug` flash message.
    pub fn debug(self, message: impl Into<String>) -> Self {
        self.push(Level::Debug, message)
    }

    /// Push an `Info` flash message.
    pub fn info(self, message: impl Into<String>) -> Self {
        self.push(Level::Info, message)
    }

    /// Push an `Success` flash message.
    pub fn success(self, message: impl Into<String>) -> Self {
        self.push(Level::Success, message)
    }

    /// Push an `Warning` flash message.
    pub fn warning(self, message: impl Into<String>) -> Self {
        self.push(Level::Warning, message)
    }

    /// Push an `Error` flash message.
    pub fn error(self, message: impl Into<String>) -> Self {
        self.push(Level::Error, message)
    }

    /// Push a flash message with the given level and message.
    pub fn push(mut self, level: Level, message: impl Into<String>) -> Self {
        self.flashes.push(FlashMessage {
            message: message.into(),
            level,
        });
        self
    }
}

impl<S> FromRequestParts<S> for Flash
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);

        Ok(Self {
            key: config.key,
            use_secure_cookies: config.use_secure_cookies,
            flashes: Default::default(),
        })
    }
}

const COOKIE_NAME: &str = "axum-flash";

impl IntoResponseParts for Flash {
    type Error = Infallible;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let json =
            serde_json::to_string(&self.flashes).expect("failed to serialize flash messages");

        let cookies = SignedCookieJar::new(self.key.clone());

        let cookies = cookies.add(create_cookie(json, self.use_secure_cookies));
        cookies.into_response_parts(res)
    }
}

pub(crate) fn create_cookie(
    value: impl Into<Cow<'static, str>>,
    use_secure_cookies: bool,
) -> Cookie<'static> {
    // process is inspired by
    // https://github.com/LukeMathWalker/actix-web-flash-messages/blob/main/src/storage/cookies.rs#L54
    Cookie::build((COOKIE_NAME, value))
        // only send the cookie for https (maybe)
        .secure(use_secure_cookies)
        // don't allow javascript to access the cookie
        .http_only(true)
        // don't send the cookie to other domains
        .same_site(cookie::SameSite::Strict)
        // allow the cookie for all paths
        .path("/")
        // expire after 10 minutes
        .max_age(
            Duration::from_secs(10 * 60)
                .try_into()
                .expect("failed to convert `std::time::Duration` to `time::Duration`"),
        )
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashMessage {
    #[serde(rename = "l")]
    pub level: Level,
    #[serde(rename = "m")]
    pub message: String,
}

/// Verbosity level of a flash message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    #[allow(missing_docs)]
    Debug = 0,
    #[allow(missing_docs)]
    Info = 1,
    #[allow(missing_docs)]
    Success = 2,
    #[allow(missing_docs)]
    Warning = 3,
    #[allow(missing_docs)]
    Error = 4,
}

/// Allow the level to be printed or rendered as text
impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Debug => write!(f, "debug"),
            Level::Info => write!(f, "info"),
            Level::Success => write!(f, "success"),
            Level::Warning => write!(f, "warning"),
            Level::Error => write!(f, "error"),
        }
    }
}

/// Configuration for axum-flash.
#[derive(Clone)]
pub struct Config {
    use_secure_cookies: bool,
    key: Key,
}

impl Config {
    /// Create a new `Config` using the given key.
    ///
    /// Cookies will be signed using `key`.
    pub fn new(key: Key) -> Self {
        Self {
            use_secure_cookies: true,
            key,
        }
    }

    /// Mark the cookie as secure so the cookie will only be sent on `https`.
    ///
    /// Defaults to marking cookies as secure.
    ///
    /// For local development, depending on your brwoser, you might have to set
    /// this to `false` for flash messages to show up.
    ///
    /// See [mdn] for more details on secure cookies.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie
    pub fn use_secure_cookies(mut self, use_secure_cookies: bool) -> Self {
        self.use_secure_cookies = use_secure_cookies;
        self
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("use_secure_cookies", &self.use_secure_cookies)
            .field("key", &"REDACTED")
            .finish()
    }
}

/// Extractor for incoming flash messages.
///
/// See [root module docs](crate) for an example.
#[derive(Clone)]
pub struct IncomingFlashes {
    pub flashes: Vec<FlashMessage>,
    use_secure_cookies: bool,
    key: Key,
}

impl fmt::Debug for IncomingFlashes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IncomingFlashes")
            .field("flashes", &self.flashes)
            .field("use_secure_cookies", &self.use_secure_cookies)
            .field("key", &"REDACTED")
            .finish()
    }
}

impl IncomingFlashes {
    /// Get an iterator over the flash messages.
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.flashes.iter())
    }

    /// Get the number of flash messages.
    pub fn len(&self) -> usize {
        self.flashes.len()
    }

    /// Whether there are any flash messages or not.
    pub fn is_empty(&self) -> bool {
        self.flashes.is_empty()
    }
}

/// An iterator over the flash messages.
#[derive(Debug)]
pub struct Iter<'a>(std::slice::Iter<'a, FlashMessage>);

impl<'a> Iterator for Iter<'a> {
    type Item = (Level, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let message = self.0.next()?;
        Some((message.level, &message.message))
    }
}

impl<'a> IntoIterator for &'a IncomingFlashes {
    type Item = (Level, &'a str);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<S> FromRequestParts<S> for IncomingFlashes
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);
        let cookies = SignedCookieJar::from_headers(&parts.headers, config.key.clone());

        let flashes = cookies
            .get(COOKIE_NAME)
            .map(|cookie| cookie.into_owned())
            .and_then(|cookie| serde_json::from_str::<Vec<FlashMessage>>(cookie.value()).ok())
            .unwrap_or_default();

        Ok(Self {
            flashes,
            use_secure_cookies: config.use_secure_cookies,
            key: config.key,
        })
    }
}

impl IntoResponseParts for IncomingFlashes {
    type Error = Infallible;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let cookies = SignedCookieJar::from_headers(res.headers(), self.key);

        let mut cookie = create_cookie("".to_owned(), self.use_secure_cookies);
        cookie.make_removal();
        let cookies = cookies.add(cookie);
        cookies.into_response_parts(res)
    }
}

impl IntoResponse for IncomingFlashes {
    fn into_response(self) -> Response {
        (self, ()).into_response()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, header},
        response::Redirect,
        routing::get,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn basic() {
        let config = Config::new(Key::generate()).use_secure_cookies(false);

        let app = Router::new()
            .route("/", get(root))
            .route("/set-flash", get(set_flash))
            .with_state(config);

        async fn root(flash: IncomingFlashes) -> (IncomingFlashes, String) {
            let messages = flash
                .into_iter()
                .map(|(level, text)| format!("{:?}: {}", level, text))
                .collect::<Vec<_>>()
                .join(", ");
            (flash, messages)
        }

        #[axum::debug_handler(state = Config)]
        async fn set_flash(flash: Flash) -> (Flash, Redirect) {
            (flash.debug("Hi from flash!"), Redirect::to("/"))
        }

        let request = Request::builder()
            .uri("/set-flash")
            .body(Body::empty())
            .unwrap();
        let mut response = app.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_redirection());
        let cookie = response.headers_mut().remove(header::SET_COOKIE).unwrap();

        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();

        assert!(
            response.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .contains("Max-Age=0"),
        );

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Debug: Hi from flash!");
    }
}
