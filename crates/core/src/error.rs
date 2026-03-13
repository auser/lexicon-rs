use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum CoreError {
    #[error("spec error: {0}")]
    Spec(#[from] lexicon_spec::error::SpecError),

    #[error("repo error: {0}")]
    Repo(#[from] lexicon_repo::error::RepoError),

    #[error("scaffold error: {0}")]
    Scaffold(#[from] lexicon_scaffold::error::ScaffoldError),

    #[error("conversation error: {0}")]
    Conversation(#[from] lexicon_conversation::error::ConversationError),

    #[error("audit error: {0}")]
    Audit(#[from] lexicon_audit::error::AuditError),

    #[error("gates error: {0}")]
    Gates(#[from] lexicon_gates::error::GatesError),

    #[error("scoring error: {0}")]
    Scoring(#[from] lexicon_scoring::error::ScoringError),

    #[error("fs error: {0}")]
    Fs(#[from] lexicon_fs::error::FsError),

    #[error("{0}")]
    Other(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
