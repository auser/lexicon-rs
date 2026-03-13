use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("workflow aborted by user")]
    Aborted,

    #[error("workflow step failed: {message}")]
    StepFailed { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("fs error: {0}")]
    Fs(#[from] lexicon_fs::error::FsError),

    #[error("dialoguer error: {0}")]
    Dialoguer(String),
}

pub type ConversationResult<T> = Result<T, ConversationError>;
