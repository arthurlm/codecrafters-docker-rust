use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ContainerError {
    #[error("HTTP: {0}")]
    Http(String),

    #[error("Unhandled status code: {0}")]
    UnhandledStatusCode(StatusCode),

    #[error("Auth error: {0}")]
    Auth(&'static str),

    #[error("Unsupported manifest file: {0}")]
    Manifest(&'static str),
}

impl From<reqwest::Error> for ContainerError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}
