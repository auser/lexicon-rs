use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum RepoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not a valid repository root: {path}")]
    NotARepo { path: String },

    #[error("lexicon not initialized in {path} — run `lexicon init` first")]
    NotInitialized { path: String },

    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("spec error: {0}")]
    Spec(#[from] crate::spec::error::SpecError),

    #[error("fs error: {0}")]
    Fs(#[from] crate::fs::error::FsError),
}

pub type RepoResult<T> = Result<T, RepoError>;
