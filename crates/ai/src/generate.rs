//! AI-powered artifact generation.
//!
//! Combines the AI client, prompt builder, and context to generate artifacts
//! from natural language intent.

use lexicon_repo::layout::RepoLayout;

use crate::boundary::AiProvider;
use crate::context::assemble_context;
use crate::error::AiResult;
use crate::prompt::{ArtifactKind, intent_prompt, system_prompt};

/// A generated artifact ready for preview and approval.
#[derive(Debug, Clone)]
pub struct GeneratedArtifact {
    /// The kind of artifact generated.
    pub kind: ArtifactKind,
    /// Suggested file path (relative to repo root).
    pub path: String,
    /// The generated content.
    pub content: String,
    /// Format hint for rendering ("toml", "rust", "markdown").
    pub format: String,
}

/// Result of artifact generation, including any context warnings.
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// The generated artifact.
    pub artifact: GeneratedArtifact,
    /// Warnings from context loading (e.g. missing manifest, no contracts).
    pub warnings: Vec<String>,
}

/// Generate an artifact from a natural language intent.
pub fn generate_artifact(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    kind: ArtifactKind,
    intent: &str,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(kind);
    let user_msg = intent_prompt(kind, intent, &context);

    let content = provider.complete(system, &user_msg)?;

    let slug = slugify(intent);
    let (path, format) = match kind {
        ArtifactKind::Contract => (format!("specs/contracts/{slug}.toml"), "toml".to_string()),
        ArtifactKind::Conformance => {
            (format!("tests/conformance/{slug}.rs"), "rust".to_string())
        }
        ArtifactKind::Behavior => (format!("specs/behavior/{slug}.md"), "markdown".to_string()),
        ArtifactKind::Improve => ("".to_string(), "markdown".to_string()),
    };

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind,
            path,
            content,
            format,
        },
        warnings,
    })
}

/// Generate multiple artifacts (contract + conformance + behavior) from a single intent.
pub fn generate_multi_artifact(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    intent: &str,
) -> AiResult<Vec<GenerateResult>> {
    let kinds = [
        ArtifactKind::Contract,
        ArtifactKind::Conformance,
        ArtifactKind::Behavior,
    ];

    let mut results = Vec::new();
    for kind in kinds {
        results.push(generate_artifact(provider, layout, kind, intent)?);
    }
    Ok(results)
}

/// Generate improvement suggestions for the repository.
pub fn generate_improvements(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    goal: Option<&str>,
) -> AiResult<(String, Vec<String>)> {
    let (context, warnings) = load_context(layout);
    let artifact_summary = "";
    let user_msg = crate::prompt::improve_prompt(&context, artifact_summary, goal);
    let system = system_prompt(ArtifactKind::Improve);

    let result = provider.complete(system, &user_msg)?;
    Ok((result, warnings))
}

/// Load repo context for AI prompts. Returns the context string and any warnings.
pub fn load_context(layout: &RepoLayout) -> (String, Vec<String>) {
    let mut warnings = Vec::new();

    let manifest_path = layout.manifest_path();
    let manifest = match std::fs::read_to_string(&manifest_path) {
        Ok(text) => match toml::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                warnings.push(format!("Failed to parse manifest: {e}"));
                return (String::new(), warnings);
            }
        },
        Err(_) => {
            warnings.push("No manifest found — AI will have limited context".to_string());
            return (String::new(), warnings);
        }
    };

    let contracts_dir = layout.contracts_dir();
    let contracts = load_contracts(&contracts_dir);
    if contracts.is_empty() {
        warnings.push("No contracts found — AI context will lack contract details".to_string());
    }

    let score_path = layout.scoring_model_path();
    let score_model = std::fs::read_to_string(&score_path)
        .ok()
        .and_then(|t| toml::from_str(&t).ok());

    let gates_path = layout.gates_path();
    let gates_model = std::fs::read_to_string(&gates_path)
        .ok()
        .and_then(|t| toml::from_str(&t).ok());

    let ctx = assemble_context(&manifest, &contracts, score_model.as_ref(), gates_model.as_ref());
    (ctx, warnings)
}

fn load_contracts(dir: &std::path::Path) -> Vec<lexicon_spec::contract::Contract> {
    let mut contracts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "toml") {
                if let Ok(text) = std::fs::read_to_string(entry.path()) {
                    if let Ok(c) = toml::from_str(&text) {
                        contracts.push(c);
                    }
                }
            }
        }
    }
    contracts
}

/// Convert intent text to a filesystem-safe slug.
fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Key Value Store"), "key-value-store");
        assert_eq!(slugify("async key-value store with TTL"), "async-key-value-store-with-ttl");
        assert_eq!(slugify("  hello   world  "), "hello-world");
    }
}
