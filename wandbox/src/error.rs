use err_derive::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "network error: {}", _0)]
    Network(#[error(source)] reqwest::Error),
    #[error(display = "JSON error: {}", _0)]
    JSON(#[error(source)] serde_json::Error),
    #[error(display = "Parse URL error: {}", _0)]
    URL(#[error(source)] url::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
