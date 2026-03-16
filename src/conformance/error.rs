use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConformanceError {
    #[error("contract not found: {id}")]
    ContractNotFound { id: String },

    #[error("no invariants defined in contract {id} — nothing to generate")]
    NoInvariants { id: String },
}

pub type ConformanceResult<T> = Result<T, ConformanceError>;
