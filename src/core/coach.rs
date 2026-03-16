//! Interactive AI-assisted artifact coaching loop.
//!
//! Coach mode provides a conversational refinement loop where users describe
//! intent, AI generates artifact drafts, and users can accept, refine, or reject.

use crate::ai::generate::GeneratedArtifact;
use crate::ai::prompt::ArtifactKind;
use crate::conversation::driver::{ConversationDriver, ProposalResponse};
use crate::conversation::session::save_session;
use crate::conversation::workflow::{Proposal, Question};
use crate::repo::layout::RepoLayout;
use crate::spec::common::{StepType, WorkflowKind};
use crate::spec::session::ConversationSession;

use super::error::CoreResult;
use super::generate::{accept_artifact, generate_from_intent, refine_from_intent, reject_artifact};

/// Maximum refinement iterations before stopping.
const MAX_REFINEMENTS: usize = 10;

/// What the user wants to coach.
pub enum CoachTarget {
    /// Draft a contract from a description.
    Contract { description: String },
    /// Draft conformance tests for an existing contract.
    Conformance { contract_id: String },
    /// Draft implementation prompts for a contract.
    Prompt {
        contract_id: String,
        targets: Option<String>,
    },
    /// General improvement suggestions.
    Improve,
    /// Open-ended: ask the user what they want.
    OpenEnded,
}

/// Result of a coaching session.
pub struct CoachResult {
    pub accepted: Vec<GeneratedArtifact>,
}

/// Run the coach loop.
///
/// The loop generates an artifact draft from the user's intent, presents it
/// for review, and supports iterative refinement via AI before final acceptance.
pub fn run_coach(
    layout: &RepoLayout,
    target: CoachTarget,
    driver: &dyn ConversationDriver,
) -> CoreResult<CoachResult> {
    let mut session = ConversationSession::new(WorkflowKind::Coach);
    let mut accepted = Vec::new();

    let (kind, intent) = resolve_target(layout, target, driver, &mut session)?;

    session.add_step(StepType::UserInput, format!("Intent: {intent}"));

    let label = kind_label(kind);
    driver.present_info(&format!("Generating {label} draft..."));

    let result = generate_from_intent(layout, kind, &intent)?;

    for w in &result.warnings {
        driver.present_info(&format!("Warning: {w}"));
    }

    let mut artifact = result.artifact;
    session.add_step(StepType::Proposal, format!("Generated: {}", artifact.path));

    // Refinement loop
    for iteration in 0..MAX_REFINEMENTS {
        let proposal = Proposal {
            title: format!("Draft {} (revision {})", kind_label(kind), iteration + 1),
            content: artifact.content.clone(),
            format: artifact.format.clone(),
        };

        match driver.present_proposal(&proposal)? {
            ProposalResponse::Accept => {
                accept_artifact(layout, &artifact)?;
                session.add_step(StepType::Write, format!("Accepted: {}", artifact.path));
                session.add_decision(
                    "outcome".to_string(),
                    "accepted".to_string(),
                    Some(artifact.path.clone()),
                );
                accepted.push(artifact);
                break;
            }
            ProposalResponse::Refine(feedback) => {
                session.add_step(StepType::Refinement, feedback.clone());
                driver.present_info("Refining draft based on feedback...");

                let refined =
                    refine_from_intent(layout, kind, &intent, &artifact.content, &feedback)?;

                for w in &refined.warnings {
                    driver.present_info(&format!("Warning: {w}"));
                }

                artifact = refined.artifact;
                session.add_step(
                    StepType::Proposal,
                    format!("Refined: {}", artifact.path),
                );
            }
            ProposalResponse::Skip | ProposalResponse::Abort => {
                reject_artifact(layout, &artifact)?;
                session.add_decision(
                    "outcome".to_string(),
                    "rejected".to_string(),
                    None,
                );
                break;
            }
        }

        if iteration == MAX_REFINEMENTS - 1 {
            driver.present_info("Maximum refinement iterations reached.");
        }
    }

    // Update prompt graph if we accepted a source artifact
    if !accepted.is_empty() {
        update_prompt_graph_if_needed(layout, driver);
    }

    // Save session
    let artifact_id = accepted.first().map(|a| a.path.clone());
    session.complete(artifact_id);
    let _ = save_session(&layout.conversations_dir(), &session);

    Ok(CoachResult { accepted })
}

/// Resolve a CoachTarget into an (ArtifactKind, intent) pair.
fn resolve_target(
    layout: &RepoLayout,
    target: CoachTarget,
    driver: &dyn ConversationDriver,
    session: &mut ConversationSession,
) -> CoreResult<(ArtifactKind, String)> {
    match target {
        CoachTarget::Contract { description } => Ok((ArtifactKind::Contract, description)),

        CoachTarget::Conformance { contract_id } => {
            let contract = load_contract(layout, &contract_id)?;
            let intent = format!(
                "Generate conformance tests for the \"{}\" contract. Scope: {}",
                contract.title, contract.scope
            );
            Ok((ArtifactKind::Conformance, intent))
        }

        CoachTarget::Prompt {
            contract_id,
            targets,
        } => {
            let contract = load_contract(layout, &contract_id)?;
            let mut intent = format!(
                "Generate implementation prompt for the \"{}\" contract. Scope: {}",
                contract.title, contract.scope
            );
            if let Some(t) = targets {
                intent.push_str(&format!(". Target backends: {t}"));
            }
            Ok((ArtifactKind::ImplementationPrompt, intent))
        }

        CoachTarget::Improve => {
            let goal = driver.present_question(&Question::simple(
                "What would you like to improve? (e.g. coverage, contracts, architecture)",
            ))?;
            session.add_step(StepType::UserInput, goal.clone());
            Ok((ArtifactKind::Improve, goal))
        }

        CoachTarget::OpenEnded => {
            let kind_str = driver.present_question(&Question::simple(
                "What would you like to create? (contract, conformance, prompt, improve)",
            ))?;
            session.add_step(StepType::UserInput, kind_str.clone());

            let kind = match kind_str.trim().to_lowercase().as_str() {
                "contract" => ArtifactKind::Contract,
                "conformance" => ArtifactKind::Conformance,
                "prompt" => ArtifactKind::ImplementationPrompt,
                "improve" => ArtifactKind::Improve,
                _ => ArtifactKind::Contract, // default
            };

            let intent = driver.present_question(&Question::simple("Describe what you want:"))?;
            session.add_step(StepType::UserInput, intent.clone());

            Ok((kind, intent))
        }
    }
}

/// Load a contract by ID from the repo.
fn load_contract(
    layout: &RepoLayout,
    contract_id: &str,
) -> CoreResult<crate::spec::contract::Contract> {
    let path = layout.contracts_dir().join(format!("{contract_id}.toml"));
    let text = std::fs::read_to_string(&path).map_err(|_| {
        super::error::CoreError::Other(format!("Contract not found: {contract_id}"))
    })?;
    let contract: crate::spec::contract::Contract = toml::from_str(&text)
        .map_err(|e| super::error::CoreError::Other(format!("Failed to parse contract: {e}")))?;
    Ok(contract)
}

/// Best-effort prompt graph update after accepting an artifact.
fn update_prompt_graph_if_needed(layout: &RepoLayout, driver: &dyn ConversationDriver) {
    if let Ok(mut graph) = super::prompt_graph::load_graph(layout) {
        if let Ok(nodes) = super::prompt_graph::discover_source_nodes(layout) {
            for node in nodes {
                super::prompt_graph::upsert_node(&mut graph, node);
            }
            let _ = super::prompt_graph::save_graph(layout, &graph);
        }

        // Check for stale prompts
        if let Ok(dirty) = super::prompt_graph::find_dirty_sources(layout, &graph) {
            if !dirty.is_empty() {
                let affected = super::prompt_graph::find_affected_prompts(&graph, &dirty);
                if !affected.is_empty() {
                    driver.present_info(&format!(
                        "{} implementation prompt(s) are now stale. Run `lexicon prompt regenerate` to update.",
                        affected.len()
                    ));
                }
            }
        }
    }
}

fn kind_label(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Contract => "contract",
        ArtifactKind::Conformance => "conformance test suite",
        ArtifactKind::Behavior => "behavior scenario",
        ArtifactKind::Improve => "improvement suggestions",
        ArtifactKind::PropertyTest => "property test suite",
        ArtifactKind::Fuzz => "fuzz test harness",
        ArtifactKind::EdgeCase => "edge case test suite",
        ArtifactKind::InferContract => "inferred contract",
        ArtifactKind::ImplementationPrompt => "implementation prompt",
    }
}
