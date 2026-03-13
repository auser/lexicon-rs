use std::io;

use thiserror::Error;

/// Errors that can occur during coverage analysis.
#[derive(Debug, Error)]
pub enum CoverageError {
    /// An I/O error occurred while reading files.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Failed to parse a file or data structure.
    #[error("parse error: {0}")]
    Parse(String),

    /// No contracts were found to analyze.
    #[error("no contracts found")]
    NoContracts,
}
