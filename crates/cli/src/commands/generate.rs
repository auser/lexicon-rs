use lexicon_ai::prompt::ArtifactKind;
use lexicon_core::generate::generate_from_intent;
use lexicon_repo::layout::RepoLayout;

use crate::commands::review::{review_artifact, show_warnings};
use crate::output;

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
        ArtifactKind::ImplementationPrompt => "implementation prompt",
    };

    output::info(&format!("Generating {kind_label}..."));

    let result = generate_from_intent(layout, kind, intent)?;
    show_warnings(&result.warnings);
    review_artifact(layout, &result.artifact)?;

    Ok(())
}
