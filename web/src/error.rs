use color_eyre::eyre;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Enumerate any possible app arrors here.
    ///
    /// Return `500 Internal Server Error` on a `eyre::Error`.
    #[error("an internal server error occured")]
    Unexpected(#[from] eyre::Error),
}
