use thiserror::Error;

use crate::version::SchemaVersion;

/// Errors that can occur when working with lexicon spec types.
#[derive(Debug, Error)]
pub enum SpecError {
    #[error("incompatible schema version: found {found}, expected compatible with {expected}")]
    IncompatibleVersion {
        found: SchemaVersion,
        expected: SchemaVersion,
    },

    #[error("validation error: {message}")]
    Validation { message: String },

    #[error("missing required field: {field}")]
    MissingField { field: String },

    #[error("duplicate id: {id}")]
    DuplicateId { id: String },

    #[error("invalid contract id: {id} — must be a non-empty kebab-case slug")]
    InvalidContractId { id: String },

    #[error("parse error: {0}")]
    Parse(String),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for spec operations.
pub type SpecResult<T> = Result<T, SpecError>;
