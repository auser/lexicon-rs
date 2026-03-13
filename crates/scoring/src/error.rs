use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScoringError {
    #[error("no dimensions defined in score model")]
    NoDimensions,

    #[error("dimension {id} has zero weight")]
    ZeroWeight { id: String },

    #[error("missing result for dimension: {id}")]
    MissingResult { id: String },
}

pub type ScoringResult<T> = Result<T, ScoringError>;
