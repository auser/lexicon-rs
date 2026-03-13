use thiserror::Error;

/// Errors that can occur during file operations.
#[derive(Debug, Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("file not found: {path}")]
    NotFound { path: String },

    #[error("managed block not found: marker={marker}")]
    ManagedBlockNotFound { marker: String },

    #[error("managed block markers are malformed in {path}")]
    MalformedManagedBlock { path: String },

    #[error("backup failed for {path}: {reason}")]
    BackupFailed { path: String, reason: String },
}

pub type FsResult<T> = Result<T, FsError>;
