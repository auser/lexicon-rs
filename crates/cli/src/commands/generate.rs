use dialoguer::Select;
use lexicon_ai::prompt::ArtifactKind;
use lexicon_core::generate::{accept_artifact, generate_from_intent, reject_artifact};
use lexicon_repo::layout::RepoLayout;

use crate::output;

/// Run AI artifact generation from natural language intent.
pub fn run(intent: &str) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("AI Artifact Generation");
    output::info(&format!("Intent: {intent}"));
    output::divider();

    // Generate contract as the default artifact type for the top-level generate command
    run_generate(&layout, ArtifactKind::Contract, intent)
}

/// Generate a specific artifact type from intent.
pub fn run_generate(
    layout: &RepoLayout,
    kind: ArtifactKind,
    intent: &str,
) -> miette::Result<()> {
    let kind_label = match kind {
        ArtifactKind::Contract => "contract",
        ArtifactKind::Conformance => "conformance tests",
        ArtifactKind::Behavior => "behavior scenarios",
        ArtifactKind::Improve => "improvements",
    };

    output::info(&format!("Generating {kind_label}..."));

    let artifact = generate_from_intent(layout, kind, intent)?;

    // Patch preview
    println!();
    output::heading("Patch Preview");
    output::info(&format!("File: {}", artifact.path));
    output::divider();
    println!("{}", &artifact.content);
    output::divider();

    // Accept / Edit / Reject
    let choices = &["Accept", "Reject"];
    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(choices)
        .default(0)
        .interact()
        .map_err(|e| miette::miette!("selection error: {e}"))?;

    match selection {
        0 => {
            accept_artifact(layout, &artifact)?;
            output::success(&format!("Artifact written to {}", artifact.path));
        }
        _ => {
            reject_artifact(layout, &artifact)?;
            output::warning("Artifact rejected");
        }
    }

    Ok(())
}

/// Run the AI improvement flow.
pub fn run_improve(goal: Option<&str>) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("AI-Guided Improvement");
    if let Some(g) = goal {
        output::info(&format!("Goal: {g}"));
    }
    output::divider();

    output::info("Analyzing repository...");

    let suggestions = lexicon_core::generate::generate_improve(&layout, goal)?;

    output::heading("Improvement Suggestions");
    output::divider();
    println!("{suggestions}");
    output::divider();

    Ok(())
}
