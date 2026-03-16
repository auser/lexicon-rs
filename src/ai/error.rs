use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("AI provider not available")]
    NotAvailable,

    #[error("AI request failed: {reason}")]
    RequestFailed { reason: String },

    #[error("rate limited: {message}")]
    RateLimited {
        message: String,
        retry_after_secs: Option<u64>,
    },

    #[error("policy violation: {message}")]
    PolicyViolation { message: String },
}

pub type AiResult<T> = Result<T, AiError>;
