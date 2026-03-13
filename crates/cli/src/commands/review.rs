use dialoguer::{Editor, Select};
use lexicon_core::generate::{accept_artifact, reject_artifact};
use lexicon_repo::layout::RepoLayout;

use crate::output;

/// Show any context warnings to the user.
pub(crate) fn show_warnings(warnings: &[String]) {
    for w in warnings {
        output::warning(w);
    }
}

/// Present a patch preview and let the user accept, edit, or reject.
pub(crate) fn review_artifact(
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
