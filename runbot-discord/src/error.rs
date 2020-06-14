use std::{io, result, str};

use err_derive::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "{}", _0)]
    Runbot(#[error(cause)] runbot::Error),
    #[error(display = "Discord error: {}", _0)]
    Discord(#[error(cause)] serenity::Error),
    #[error(display = "encoding error: {}", _0)]
    Encoding(#[error(cause)] str::Utf8Error),
    #[error(display = "IO error: {}", _0)]
    IO(#[error(source)] io::Error),
}

impl<W> From<io::IntoInnerError<W>> for Error {
    fn from(e: io::IntoInnerError<W>) -> Error {
        Error::IO(e.into())
    }
}

pub type Result<T> = result::Result<T, Error>;
