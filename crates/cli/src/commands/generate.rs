use dialoguer::{Editor, Select};
use lexicon_ai::prompt::ArtifactKind;
use lexicon_core::generate::{accept_artifact, generate_from_intent, generate_multi, reject_artifact};
use lexicon_repo::layout::RepoLayout;

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

/// Show any context warnings to the user.
fn show_warnings(warnings: &[String]) {
    for w in warnings {
        output::warning(w);
    }
}

/// Present a patch preview and let the user accept, edit, or reject.
fn review_artifact(
    layout: &RepoLayout,
    artifact: &lexicon_ai::generate::GeneratedArtifact,
) -> miette::Result<()> {
    println!();
    output::heading("Patch Preview");
    output::info(&format!("File: {}", artifact.path));
    output::divider();
    println!("{}", &artifact.content);
    output::divider();

    let choices = &["Accept", "Edit", "Reject"];
    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(choices)
        .default(0)
        .interact()
        .map_err(|e| miette::miette!("selection error: {e}"))?;

    match selection {
        0 => {
            accept_artifact(layout, artifact)?;
            output::success(&format!("Artifact written to {}", artifact.path));
        }
        1 => {
            let edited = Editor::new()
                .extension(match artifact.format.as_str() {
                    "toml" => ".toml",
                    "rust" => ".rs",
                    "markdown" => ".md",
                    _ => ".txt",
                })
                .edit(&artifact.content)
                .map_err(|e| miette::miette!("editor error: {e}"))?;

            if let Some(content) = edited {
                let mut edited_artifact = artifact.clone();
                edited_artifact.content = content;
                accept_artifact(layout, &edited_artifact)?;
                output::success(&format!("Edited artifact written to {}", artifact.path));
            } else {
                output::warning("No changes made, artifact not saved");
            }
        }
        _ => {
            reject_artifact(layout, artifact)?;
            output::warning("Artifact rejected");
        }
    }

    Ok(())
}
