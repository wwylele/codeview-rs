use std::num::TryFromIntError;
use thiserror::Error;

/// An error that occurred when generating CodeView data
#[derive(Error, Debug)]
pub enum Error<WriteErrorType: std::error::Error + 'static> {
    #[error("SectionWrite reported error")]
    WriteError(WriteErrorType),

    #[error("Integer overflow")]
    IntError(#[from] TryFromIntError),

    #[error("Failed to encode string")]
    StringError(String),
}

pub(crate) fn wu<T, W: std::error::Error + 'static>(result: Result<T, W>) -> Result<T, Error<W>> {
    result.map_err(Error::WriteError)
}
