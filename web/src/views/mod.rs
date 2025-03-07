use crate::error::Error;

pub mod auth;
pub mod home;
pub mod todos;

/// ------------------------------------------------------------------------
/// A helper function to render a template and convert it into an HTML response.
/// ------------------------------------------------------------------------
///
/// Takes in any type that implements the `rinja::Template` trait, renders it,
///
/// # Parameters
///
/// - `tmpl` - The template to render. Must implement the `rinja::Template` trait.
///
/// # Returns
///
/// An HTML response containing the rendered template.
///
/// # Example
///
/// ```rust
/// use axum::response::IntoResponse;
/// use rinja::Template;
/// use web::views::{home::Index, html};
///
/// #[derive(Template)]
/// #[template(path = "index.html")]
/// struct HomeView {}
///
/// impl IntoResponse for HomeView {
///    fn into_response(self) -> axum::response::Response {
///       html(Index {})
///    }
/// ```
///
/// ------------------------------------------------------------------------
pub fn html<T: rinja::Template>(tmpl: T) -> axum::response::Response {
    axum::response::IntoResponse::into_response(
        tmpl.render()
            .map_err(|e| Error::ViewEngine(e.into()))
            .map(axum::response::Html),
    )
}
