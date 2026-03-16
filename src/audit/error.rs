use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("fs error: {0}")]
    Fs(#[from] crate::fs::error::FsError),
}

pub type AuditResult<T> = Result<T, AuditError>;
