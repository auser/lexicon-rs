use std::io;

/// Errors that can occur during API extraction and diffing.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] syn::Error),

    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("baseline not found: {0}")]
    BaselineNotFound(String),
}
