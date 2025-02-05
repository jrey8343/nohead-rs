use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::eyre;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occured while interacting with the database.
    ///
    /// Return `500 Internal Server Error` on a db error.
    #[error("an error occured while interacting with the database")]
    Database(#[from] nohead_rs_db::Error),
    /// Enumerate any possible app arrors here.
    ///
    /// Return `500 Internal Server Error` on a `eyre::Error`.
    #[error("Error: {0}")]
    Unexpected(#[from] eyre::Error),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Database(nohead_rs_db::Error::NoRecordFound) => StatusCode::NOT_FOUND,
            Error::Database(nohead_rs_db::Error::ValidationError(_)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Error::Database(nohead_rs_db::Error::DatabaseError(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Database(nohead_rs_db::Error::NoRecordFound) => {
                // TODO: Return a not found view here.
            }
            Error::Database(nohead_rs_db::Error::ValidationError(ref err)) => {
                // TODO: Return a validation error view here.
            }
            Error::Database(nohead_rs_db::Error::DatabaseError(ref err)) => {
                error!(
                    "an error occured while interacting with the database: {:?}",
                    err
                );
            }
            Error::Unexpected(ref err) => {
                error!("an internal server error occured: {:?}", err);
            }
        }

        // TODO: Return a defaul error view here.
        self.status_code().into_response()
    }
}
