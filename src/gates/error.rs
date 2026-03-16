use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("gate execution failed: {gate_id} — {reason}")]
    ExecutionFailed { gate_id: String, reason: String },

    #[error("gate timed out: {gate_id} after {timeout_secs}s")]
    Timeout { gate_id: String, timeout_secs: u64 },

    #[error("policy violation: cannot skip required gate {gate_id}")]
    CannotSkipRequired { gate_id: String },

    #[error("policy violation: cannot weaken gate {gate_id} without approval")]
    WeakeningDenied { gate_id: String },
}

pub type GatesResult<T> = Result<T, GatesError>;
