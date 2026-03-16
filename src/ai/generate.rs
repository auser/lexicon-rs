//! AI-powered artifact generation.
//!
//! Combines the AI client, prompt builder, and context to generate artifacts
//! from natural language intent.

use crate::repo::layout::RepoLayout;
use crate::spec::contract::Contract;

use super::boundary::AiProvider;
use super::context::{assemble_context, assemble_context_selective};
use super::error::AiResult;
use super::prompt::{
    ArtifactKind, REFINE_SYSTEM, contract_based_prompt, coverage_improve_prompt,
    infer_contract_prompt, intent_prompt, refine_prompt, system_prompt,
};

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

    let slug = if matches!(kind, ArtifactKind::Contract | ArtifactKind::InferContract) {
        extract_toml_id(&content).unwrap_or_else(|| slugify(intent))
    } else {
        slugify(intent)
    };
    let (path, format) = artifact_path_and_format(kind, &slug);

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

/// Generate conformance tests from a parsed contract.
pub fn generate_from_contract(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    contract: &Contract,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(ArtifactKind::Conformance);
    let user_msg = contract_based_prompt(contract, &context);

    let content = provider.complete(system, &user_msg)?;
    let slug = slugify(&contract.id);

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::Conformance,
            path: format!("tests/conformance/{slug}.rs"),
            content,
            format: "rust".to_string(),
        },
        warnings,
    })
}

/// Generate property tests from a contract's invariants.
pub fn generate_property_tests(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    contract: &Contract,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(ArtifactKind::PropertyTest);
    let user_msg = contract_based_prompt(contract, &context);

    let content = provider.complete(system, &user_msg)?;
    let slug = slugify(&contract.id);

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::PropertyTest,
            path: format!("tests/property/{slug}.rs"),
            content,
            format: "rust".to_string(),
        },
        warnings,
    })
}

/// Generate a fuzz test harness from a contract.
pub fn generate_fuzz_target(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    contract: &Contract,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(ArtifactKind::Fuzz);
    let user_msg = contract_based_prompt(contract, &context);

    let content = provider.complete(system, &user_msg)?;
    let slug = slugify(&contract.id);

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::Fuzz,
            path: format!("fuzz/fuzz_targets/{slug}.rs"),
            content,
            format: "rust".to_string(),
        },
        warnings,
    })
}

/// Generate edge case tests from a contract.
pub fn generate_edge_case_tests(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    contract: &Contract,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(ArtifactKind::EdgeCase);
    let user_msg = contract_based_prompt(contract, &context);

    let content = provider.complete(system, &user_msg)?;
    let slug = slugify(&contract.id);

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::EdgeCase,
            path: format!("tests/edge_cases/{slug}.rs"),
            content,
            format: "rust".to_string(),
        },
        warnings,
    })
}

/// Infer a contract from an API surface summary.
pub fn infer_contract(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    api_summary: &str,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(ArtifactKind::InferContract);
    let user_msg = infer_contract_prompt(api_summary, &context);

    let content = provider.complete(system, &user_msg)?;
    let slug = extract_toml_id(&content).unwrap_or_else(|| slugify("inferred-contract"));

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::InferContract,
            path: format!("specs/contracts/{slug}.toml"),
            content,
            format: "toml".to_string(),
        },
        warnings,
    })
}

/// Generate tests to fill coverage gaps.
pub fn generate_coverage_tests(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    coverage_gaps: &str,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let system = system_prompt(ArtifactKind::Conformance);
    let user_msg = coverage_improve_prompt(coverage_gaps, &context);

    let content = provider.complete(system, &user_msg)?;

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::Conformance,
            path: "tests/conformance/coverage_fill.rs".to_string(),
            content,
            format: "rust".to_string(),
        },
        warnings,
    })
}

/// Refine an existing artifact draft based on user feedback.
pub fn refine_artifact(
    provider: &dyn AiProvider,
    layout: &RepoLayout,
    kind: ArtifactKind,
    original_intent: &str,
    previous_draft: &str,
    feedback: &str,
) -> AiResult<GenerateResult> {
    let (context, warnings) = load_context(layout);
    let user_msg = refine_prompt(kind, original_intent, &context, previous_draft, feedback);

    let content = provider.complete(REFINE_SYSTEM, &user_msg)?;

    let slug = if matches!(kind, ArtifactKind::Contract | ArtifactKind::InferContract) {
        extract_toml_id(&content).unwrap_or_else(|| slugify(original_intent))
    } else {
        slugify(original_intent)
    };
    let (path, format) = artifact_path_and_format(kind, &slug);

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

/// Map an artifact kind to its output path and format.
fn artifact_path_and_format(kind: ArtifactKind, slug: &str) -> (String, String) {
    match kind {
        ArtifactKind::Contract => (format!("specs/contracts/{slug}.toml"), "toml".to_string()),
        ArtifactKind::Conformance => (format!("tests/conformance/{slug}.rs"), "rust".to_string()),
        ArtifactKind::Behavior => (format!("specs/behavior/{slug}.md"), "markdown".to_string()),
        ArtifactKind::Improve => ("".to_string(), "markdown".to_string()),
        ArtifactKind::PropertyTest => (format!("tests/property/{slug}.rs"), "rust".to_string()),
        ArtifactKind::Fuzz => (format!("fuzz/fuzz_targets/{slug}.rs"), "rust".to_string()),
        ArtifactKind::EdgeCase => (format!("tests/edge_cases/{slug}.rs"), "rust".to_string()),
        ArtifactKind::InferContract => {
            (format!("specs/contracts/{slug}.toml"), "toml".to_string())
        }
        ArtifactKind::ImplementationPrompt => {
            (format!("specs/prompts/{slug}.md"), "markdown".to_string())
        }
    }
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

/// Load repo context with selective contract detail for chat sessions.
///
/// Only contracts whose IDs are in `active_ids` get full detail (invariants, semantics).
/// All others get a one-line summary (id + title + scope). Pass an empty slice for
/// summary-only mode.
pub fn load_context_selective(layout: &RepoLayout, active_ids: &[&str]) -> (String, Vec<String>) {
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

    let score_path = layout.scoring_model_path();
    let score_model = std::fs::read_to_string(&score_path)
        .ok()
        .and_then(|t| toml::from_str(&t).ok());

    let gates_path = layout.gates_path();
    let gates_model = std::fs::read_to_string(&gates_path)
        .ok()
        .and_then(|t| toml::from_str(&t).ok());

    let ctx = assemble_context_selective(
        &manifest,
        &contracts,
        Some(active_ids),
        score_model.as_ref(),
        gates_model.as_ref(),
    );
    (ctx, warnings)
}

fn load_contracts(dir: &std::path::Path) -> Vec<Contract> {
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

/// Extract the `id` field from generated TOML content.
/// Returns a slugified version of the id, or None if parsing fails.
fn extract_toml_id(content: &str) -> Option<String> {
    let table: toml::Table = toml::from_str(content).ok()?;
    let id = table.get("id")?.as_str()?;
    let slug = slugify(id);
    if slug.is_empty() { None } else { Some(slug) }
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

    #[test]
    fn test_artifact_path_and_format() {
        let (path, fmt) = artifact_path_and_format(ArtifactKind::PropertyTest, "kv-store");
        assert_eq!(path, "tests/property/kv-store.rs");
        assert_eq!(fmt, "rust");

        let (path, fmt) = artifact_path_and_format(ArtifactKind::Fuzz, "kv-store");
        assert_eq!(path, "fuzz/fuzz_targets/kv-store.rs");
        assert_eq!(fmt, "rust");

        let (path, fmt) = artifact_path_and_format(ArtifactKind::EdgeCase, "kv-store");
        assert_eq!(path, "tests/edge_cases/kv-store.rs");
        assert_eq!(fmt, "rust");

        let (path, fmt) = artifact_path_and_format(ArtifactKind::InferContract, "inferred");
        assert_eq!(path, "specs/contracts/inferred.toml");
        assert_eq!(fmt, "toml");
    }

    #[test]
    fn test_extract_toml_id() {
        let toml = r#"id = "my-kv-store"
title = "My KV Store"
"#;
        assert_eq!(extract_toml_id(toml), Some("my-kv-store".to_string()));
    }

    #[test]
    fn test_extract_toml_id_with_spaces() {
        let toml = r#"id = "Async Key Value Store"
title = "test"
"#;
        assert_eq!(extract_toml_id(toml), Some("async-key-value-store".to_string()));
    }

    #[test]
    fn test_extract_toml_id_missing() {
        assert_eq!(extract_toml_id("title = \"no id here\""), None);
    }

    #[test]
    fn test_extract_toml_id_invalid_toml() {
        assert_eq!(extract_toml_id("not valid toml {{{}"), None);
    }
}
