#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Json error
    ///
    /// Return `500 Internal Server Error` on a json error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Could not render template
    ///
    /// Return `500 Internal Server Error` on a template rendering error.
    #[error("could not render template")]
    Template(#[from] minijinja::Error),
    /// Could not render component with wasm
    ///
    /// Return `500 Internal Server Error` on a component rendering error.
    #[error("error rendering component with wasm")]
    Component(#[from] extism::Error),
    /// Could not render component due to mutex poisoning
    ///
    /// Return `500 Internal Server Error` on a component rendering error.
    #[error("error rendering component as mutex poisoned")]
    Mutex,
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Path(#[from] std::path::StripPrefixError),
}
