use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::eyre;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Enumerate any possible app arrors here.
    ///
    /// Return `500 Internal Server Error` on a `eyre::Error`.
    #[error("an internal server error occured")]
    Unexpected(#[from] eyre::Error),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Unexpected(ref err) => {
                error!("an internal server error occured: {:?}", err);
            }
        }

        // TODO: Return a defaul error view here.
        self.status_code().into_response()
    }
}
