use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    Toml(#[from] toml::ser::Error),

    #[error("fs error: {0}")]
    Fs(#[from] lexicon_fs::error::FsError),

    #[error("repo error: {0}")]
    Repo(#[from] lexicon_repo::error::RepoError),

    #[error("already initialized at {path}")]
    AlreadyInitialized { path: String },
}

pub type ScaffoldResult<T> = Result<T, ScaffoldError>;
