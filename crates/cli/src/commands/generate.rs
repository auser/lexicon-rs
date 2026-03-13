use lexicon_ai::prompt::ArtifactKind;
use lexicon_core::generate::{generate_from_intent, generate_multi};
use lexicon_repo::layout::RepoLayout;

use crate::commands::review::{review_artifact, show_warnings};
use crate::output;

/// Run AI artifact generation from natural language intent.
/// Generates multiple artifacts (contract + conformance + behavior) from a single intent.
pub fn run(intent: &str) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("AI Artifact Generation");
    output::info(&format!("Intent: {intent}"));
    output::divider();

    output::info("Generating contract, conformance tests, and behavior scenarios...");

    let results = generate_multi(&layout, intent)?;

    for result in &results {
        show_warnings(&result.warnings);
        review_artifact(&layout, &result.artifact)?;
    }

    Ok(())
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
        ArtifactKind::PropertyTest => "property tests",
        ArtifactKind::Fuzz => "fuzz targets",
        ArtifactKind::EdgeCase => "edge case tests",
        ArtifactKind::InferContract => "inferred contract",
    };

    output::info(&format!("Generating {kind_label}..."));

    let result = generate_from_intent(layout, kind, intent)?;
    show_warnings(&result.warnings);
    review_artifact(layout, &result.artifact)?;

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

    let (suggestions, warnings) = lexicon_core::generate::generate_improve(&layout, goal)?;
    show_warnings(&warnings);

    output::heading("Improvement Suggestions");
    output::divider();
    println!("{suggestions}");
    output::divider();

    Ok(())
}
