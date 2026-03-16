//! AI-powered artifact generation with patch preview and approval.
//!
//! This module integrates the auth system (OAuth credentials) with the AI
//! client to generate artifacts from natural language intent, then presents
//! them for user review.

use std::path::Path;

use crate::ai::boundary::AiProvider;
use crate::ai::client::ClaudeClient;
use crate::ai::generate::{
    GenerateResult, GeneratedArtifact, generate_artifact, generate_coverage_tests,
    generate_edge_case_tests, generate_from_contract, generate_fuzz_target,
    generate_property_tests, infer_contract,
    refine_artifact,
};
use crate::ai::prompt::ArtifactKind;
use crate::api::extract::extract_from_dir;
use crate::audit::writer::write_audit_record;
use crate::repo::layout::RepoLayout;
use crate::spec::audit::AuditRecord;
use crate::spec::auth::Provider;
use crate::spec::common::{Actor, AuditAction};
use crate::spec::contract::Contract;

use super::error::{CoreError, CoreResult};

/// Generate an artifact from intent, authenticate, call AI, and return the result.
///
/// The caller (CLI) is responsible for presenting the preview and collecting
/// the user's accept/reject decision. Returns the artifact and any context warnings.
pub fn generate_from_intent(
    layout: &RepoLayout,
    kind: ArtifactKind,
    intent: &str,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;
    let result = generate_artifact(&*provider, layout, kind, intent)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Generate conformance tests from an existing contract.
pub fn generate_tests_from_contract(
    layout: &RepoLayout,
    contract: &Contract,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;
    let result = generate_from_contract(&*provider, layout, contract)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Generate property tests from a contract's invariants.
pub fn generate_contract_property_tests(
    layout: &RepoLayout,
    contract: &Contract,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;
    let result = generate_property_tests(&*provider, layout, contract)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Generate a fuzz test harness from a contract.
pub fn generate_contract_fuzz_target(
    layout: &RepoLayout,
    contract: &Contract,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;
    let result = generate_fuzz_target(&*provider, layout, contract)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Generate edge case tests from a contract.
pub fn generate_contract_edge_case_tests(
    layout: &RepoLayout,
    contract: &Contract,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;
    let result = generate_edge_case_tests(&*provider, layout, contract)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Infer a contract from the public API surface.
pub fn infer_contract_from_api(
    layout: &RepoLayout,
    source_dir: Option<&Path>,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;

    let default_dir = layout.root.join("src");
    let dir = source_dir.unwrap_or(&default_dir);
    let snapshot = extract_from_dir(dir)
        .map_err(|e| CoreError::Other(format!("API extraction failed: {e}")))?;

    let mut api_summary = String::new();
    for item in &snapshot.items {
        api_summary.push_str(&format!(
            "{:?} {} — {}\n",
            item.kind,
            item.name,
            item.signature
        ));
        if let Some(doc) = &item.doc_summary {
            api_summary.push_str(&format!("  doc: {doc}\n"));
        }
    }

    if api_summary.is_empty() {
        return Err(CoreError::Other(
            "No public API items found to infer contract from".to_string(),
        ));
    }

    let result = infer_contract(&*provider, layout, &api_summary)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(result)
}

/// Generate tests to fill coverage gaps.
pub fn generate_coverage_improvement(
    layout: &RepoLayout,
) -> CoreResult<Vec<GenerateResult>> {
    let contracts = load_contracts(layout)?;
    if contracts.is_empty() {
        return Err(CoreError::Other("No contracts found".to_string()));
    }

    let report = super::coverage::coverage_report(layout, &contracts)?;
    if report.uncovered_clauses.is_empty() {
        return Ok(Vec::new());
    }

    let mut gaps = String::new();
    for uc in &report.uncovered_clauses {
        gaps.push_str(&format!(
            "- Contract: {}, Clause: {} ({}): {}\n",
            uc.contract_id, uc.clause_id, uc.clause_type, uc.description
        ));
    }

    let provider = build_ai_provider(layout, None)?;
    let result = generate_coverage_tests(&*provider, layout, &gaps)
        .map_err(|e| CoreError::Other(format!("AI generation failed: {e}")))?;
    Ok(vec![result])
}

/// Refine an existing artifact draft based on user feedback.
pub fn refine_from_intent(
    layout: &RepoLayout,
    kind: ArtifactKind,
    intent: &str,
    previous_draft: &str,
    feedback: &str,
) -> CoreResult<GenerateResult> {
    let provider = build_ai_provider(layout, None)?;
    let result = refine_artifact(&*provider, layout, kind, intent, previous_draft, feedback)
        .map_err(|e| CoreError::Other(format!("AI refinement failed: {e}")))?;
    Ok(result)
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
        ArtifactKind::PropertyTest => AuditAction::PropertyTestCreate,
        ArtifactKind::Fuzz => AuditAction::FuzzCreate,
        ArtifactKind::EdgeCase => AuditAction::EdgeCaseCreate,
        ArtifactKind::InferContract => AuditAction::ContractInfer,
        ArtifactKind::ImplementationPrompt => AuditAction::PromptGenerate,
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

/// Load all contracts from the specs/contracts directory.
fn load_contracts(layout: &RepoLayout) -> CoreResult<Vec<Contract>> {
    let dir = layout.contracts_dir();
    let mut contracts = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir)?.flatten() {
            if entry.path().extension().is_some_and(|e| e == "toml") {
                let text = std::fs::read_to_string(entry.path())?;
                let contract: Contract = toml::from_str(&text)
                    .map_err(|e| CoreError::Other(format!("Failed to parse contract: {e}")))?;
                contracts.push(contract);
            }
        }
    }
    Ok(contracts)
}

/// Build an AI provider from stored auth credentials.
///
/// When `model` is `Some`, the given model ID overrides the default.
pub(crate) fn build_ai_provider(
    layout: &RepoLayout,
    model: Option<&str>,
) -> CoreResult<Box<dyn AiProvider>> {
    let creds = super::auth::ensure_authenticated(layout, Provider::Claude)?;
    let client = ClaudeClient::new(creds.access_token);
    let client = match model {
        Some(m) => client.with_model(m),
        None => client,
    };
    Ok(Box::new(client))
}
