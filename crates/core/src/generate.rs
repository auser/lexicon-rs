//! AI-powered artifact generation with patch preview and approval.
//!
//! This module integrates the auth system (OAuth credentials) with the AI
//! client to generate artifacts from natural language intent, then presents
//! them for user review.

use lexicon_ai::boundary::AiProvider;
use lexicon_ai::client::ClaudeClient;
use lexicon_ai::generate::{
    GenerateResult, GeneratedArtifact, generate_artifact, generate_improvements,
    generate_multi_artifact,
};
use lexicon_ai::prompt::ArtifactKind;
use lexicon_audit::writer::write_audit_record;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::auth::Provider;
use lexicon_spec::common::{Actor, AuditAction};

use crate::error::{CoreError, CoreResult};

/// Generate an artifact from intent, authenticate, call AI, and return the result.
///
/// The caller (CLI) is responsible for presenting the preview and collecting
/// the user's accept/reject decision. Returns the artifact and any context warnings.
pub fn generate_from_intent(
    layout: &RepoLayout,
    kind: ArtifactKind,
    intent: &str,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout)?;
    let result = generate_artifact(&*provider, layout, kind, intent)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Generate multiple artifacts (contract + conformance + behavior) from a single intent.
pub fn generate_multi(
    layout: &RepoLayout,
    intent: &str,
) -> CoreResult<Vec<GenerateResult>> {
    let provider = build_ai_provider(layout)?;
    let results = generate_multi_artifact(&*provider, layout, intent)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(results)
}

/// Generate improvement suggestions. Returns suggestions text and context warnings.
pub fn generate_improve(
    layout: &RepoLayout,
    goal: Option<&str>,
) -> CoreResult<(String, Vec<String>)> {
    let provider = build_ai_provider(layout)?;
    let (suggestions, warnings) = generate_improvements(&*provider, layout, goal)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok((suggestions, warnings))
}

/// Write an accepted artifact to disk and record an audit entry.
pub fn accept_artifact(layout: &RepoLayout, artifact: &GeneratedArtifact) -> CoreResult<()> {
    let full_path = layout.root.join(&artifact.path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full_path, &artifact.content)?;

    let action = match artifact.kind {
        ArtifactKind::Contract => AuditAction::ContractCreate,
        ArtifactKind::Conformance => AuditAction::ConformanceCreate,
        ArtifactKind::Behavior => AuditAction::BehaviorCreate,
        ArtifactKind::Improve => AuditAction::AiImprove,
    };

    let record = AuditRecord::new(
        action,
        Actor::Ai,
        format!("AI-generated artifact: {}", artifact.path),
    );
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(())
}

/// Record that an AI suggestion was rejected.
pub fn reject_artifact(layout: &RepoLayout, artifact: &GeneratedArtifact) -> CoreResult<()> {
    let record = AuditRecord::new(
        AuditAction::AiImproveRejected,
        Actor::User,
        format!("Rejected AI-generated artifact: {}", artifact.path),
    );
    write_audit_record(&layout.audit_dir(), &record)?;
    Ok(())
}

/// Build an AI provider from stored auth credentials.
fn build_ai_provider(layout: &RepoLayout) -> CoreResult<Box<dyn AiProvider>> {
    let creds = crate::auth::ensure_authenticated(layout, Provider::Claude)?;
    Ok(Box::new(ClaudeClient::new(creds.access_token)))
}
