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
/// - `tmpl` - The template to render.
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
/// use web::views::home::Index;
///
/// #[derive(Template)]
/// #[template(path = "index.html")]
/// struct HomeView {}
///
/// impl IntoResponse for HomeView {
///    fn into_response(self) -> axum::response::Response {
///    web::views::html(Index {})
///    }
/// ```
///
/// ------------------------------------------------------------------------
pub fn html<T: rinja::Template>(tmpl: T) -> axum::response::Response {
    axum::response::IntoResponse::into_response(
        tmpl.render()
            .map_err(crate::error::Error::Render)
            .map(axum::response::Html),
    )
}
