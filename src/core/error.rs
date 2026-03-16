use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum CoreError {
    #[error("spec error: {0}")]
    Spec(#[from] crate::spec::error::SpecError),

    #[error("repo error: {0}")]
    Repo(#[from] crate::repo::error::RepoError),

    #[error("scaffold error: {0}")]
    Scaffold(#[from] crate::scaffold::error::ScaffoldError),

    #[error("conversation error: {0}")]
    Conversation(#[from] crate::conversation::error::ConversationError),

    #[error("audit error: {0}")]
    Audit(#[from] crate::audit::error::AuditError),

    #[error("gates error: {0}")]
    Gates(#[from] crate::gates::error::GatesError),

    #[error("scoring error: {0}")]
    Scoring(#[from] crate::scoring::error::ScoringError),

    #[error("fs error: {0}")]
    Fs(#[from] crate::fs::error::FsError),

    #[error("api error: {0}")]
    Api(#[from] crate::api::error::ApiError),

    #[error("coverage error: {0}")]
    Coverage(#[from] crate::coverage::error::CoverageError),

    #[error("not authenticated for provider '{provider}' — run `lexicon auth login`")]
    NotAuthenticated { provider: String },

    #[error("authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("token refresh failed for {provider}: {reason}")]
    RefreshFailed { provider: String, reason: String },

    #[error("OAuth callback error: {0}")]
    OAuthCallback(String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
