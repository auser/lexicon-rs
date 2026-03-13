//! Prompt templates for AI artifact generation.

/// Build a system prompt for generating a specific artifact type.
pub fn system_prompt(artifact_kind: ArtifactKind) -> &'static str {
    match artifact_kind {
        ArtifactKind::Contract => CONTRACT_SYSTEM,
        ArtifactKind::Conformance => CONFORMANCE_SYSTEM,
        ArtifactKind::Behavior => BEHAVIOR_SYSTEM,
        ArtifactKind::Improve => IMPROVE_SYSTEM,
    }
}

/// Build a user message for intent-driven generation.
pub fn intent_prompt(artifact_kind: ArtifactKind, intent: &str, context: &str) -> String {
    let kind_label = match artifact_kind {
        ArtifactKind::Contract => "contract",
        ArtifactKind::Conformance => "conformance test suite",
        ArtifactKind::Behavior => "behavior scenario",
        ArtifactKind::Improve => "improvement suggestions",
    };

    let template = match artifact_kind {
        ArtifactKind::Contract => CONTRACT_TEMPLATE,
        ArtifactKind::Conformance => CONFORMANCE_TEMPLATE,
        ArtifactKind::Behavior => BEHAVIOR_TEMPLATE,
        ArtifactKind::Improve => "",
    };

    let mut msg = String::new();
    msg.push_str("## Repository Context\n");
    msg.push_str(context);
    msg.push_str("\n\n");

    if !template.is_empty() {
        msg.push_str("## Template\n");
        msg.push_str(template);
        msg.push_str("\n\n");
    }

    msg.push_str(&format!("## Task\nGenerate a {kind_label} based on the following intent:\n\n"));
    msg.push_str(intent);
    msg.push('\n');
    msg
}

/// Build a prompt for improving an existing artifact.
pub fn improve_prompt(context: &str, current_artifact: &str, goal: Option<&str>) -> String {
    let mut msg = String::new();
    msg.push_str("## Repository Context\n");
    msg.push_str(context);
    msg.push_str("\n\n## Current State\n");
    msg.push_str(current_artifact);
    msg.push_str("\n\n## Task\n");
    if let Some(g) = goal {
        msg.push_str(&format!(
            "Analyze the repository and suggest improvements focused on: {g}\n"
        ));
    } else {
        msg.push_str("Analyze the repository and suggest improvements. Prioritize:\n");
        msg.push_str("1. Fixing coverage gaps\n");
        msg.push_str("2. Strengthening contracts\n");
        msg.push_str("3. Adding conformance tests\n");
        msg.push_str("4. Refining scoring models\n");
        msg.push_str("5. Improving architecture rules\n");
    }
    msg.push_str("\nOutput each suggestion as a structured proposal.\n");
    msg
}

/// Kind of artifact to generate or improve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Contract,
    Conformance,
    Behavior,
    Improve,
}

const CONTRACT_SYSTEM: &str = "\
You are an expert at defining software contracts. Generate a TOML contract file \
following the lexicon contract schema. Include:\n\
- A clear id (kebab-case)\n\
- A descriptive title\n\
- A scope statement\n\
- Invariants with ids, descriptions, severity (required/advisory), and test_tags\n\
- Required semantics with ids, descriptions, and test_tags\n\
- Forbidden semantics with ids and descriptions\n\
- Edge cases with ids and descriptions\n\
- Examples with titles and code\n\
Output ONLY valid TOML. No markdown fences, no explanations.";

const CONFORMANCE_SYSTEM: &str = "\
You are an expert at writing Rust conformance test suites. Generate a conformance \
test file that:\n\
- Uses a trait-based harness pattern\n\
- Tests all contract invariants and required semantics\n\
- Includes edge case tests\n\
- Uses #[test] functions with descriptive names\n\
- Includes test tags as comments (// lexicon::tag(\"...\"))\n\
Output ONLY valid Rust code. No markdown fences, no explanations.";

const BEHAVIOR_SYSTEM: &str = "\
You are an expert at writing BDD-style behavior scenarios. Generate a markdown \
behavior scenario file that:\n\
- Uses Given/When/Then format\n\
- Covers the main success path\n\
- Includes failure and edge case scenarios\n\
- Links to contract clauses where applicable\n\
Output ONLY valid markdown. No extra explanations.";

const IMPROVE_SYSTEM: &str = "\
You are an expert software architect analyzing a repository for improvement \
opportunities. Identify specific, actionable improvements:\n\
- Missing contract clauses\n\
- Weak test coverage\n\
- Architecture drift\n\
- Untested APIs\n\
- Weak scoring rules\n\
- Inconsistent behavior definitions\n\
\n\
For each suggestion, output:\n\
- Type: (contract/conformance/behavior/coverage/scoring/gate)\n\
- Description: what to add or change\n\
- Priority: (high/medium/low)\n\
- Artifact: the file or section affected\n\
Be specific and concise.";

const CONTRACT_TEMPLATE: &str = r#"id = "example-contract"
title = "Example Contract"
scope = "Description of what this contract covers"
status = "draft"
stability = "experimental"
version = "0.1.0"

[[invariants]]
id = "inv-001"
description = "Description of the invariant"
severity = "required"
test_tags = ["conformance.example"]

[[required_semantics]]
id = "req-001"
description = "Description of required behavior"
test_tags = ["conformance.example"]

[[forbidden_semantics]]
id = "forbid-001"
description = "Description of forbidden behavior"
test_tags = ["safety.example"]

[[edge_cases]]
id = "edge-001"
description = "Description of edge case"

[[examples]]
title = "Basic usage"
code = """
// Example code here
"""
"#;

const CONFORMANCE_TEMPLATE: &str = r#"//! Conformance test suite for <contract-name>
//!
//! Tests that implementations satisfy the contract invariants.

/// Trait that implementations must satisfy for conformance testing.
pub trait ContractConformance {
    type Instance;
    fn create_instance() -> Self::Instance;
}

#[cfg(test)]
mod tests {
    use super::*;

    // lexicon::tag("conformance.example")
    #[test]
    fn test_invariant_example() {
        // Test implementation here
    }
}
"#;

const BEHAVIOR_TEMPLATE: &str = r#"# Behavior: <Feature Name>

## Scenario: Main success path

**Given** the system is in a valid state
**When** the user performs the action
**Then** the expected outcome occurs

## Scenario: Error handling

**Given** the system is in a valid state
**When** an invalid input is provided
**Then** an appropriate error is returned
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_prompt_includes_context_and_intent() {
        let prompt = intent_prompt(ArtifactKind::Contract, "key-value store", "project context");
        assert!(prompt.contains("project context"));
        assert!(prompt.contains("key-value store"));
        assert!(prompt.contains("contract"));
    }

    #[test]
    fn improve_prompt_with_goal() {
        let prompt = improve_prompt("ctx", "artifacts", Some("coverage"));
        assert!(prompt.contains("coverage"));
        assert!(prompt.contains("ctx"));
    }

    #[test]
    fn improve_prompt_without_goal() {
        let prompt = improve_prompt("ctx", "artifacts", None);
        assert!(prompt.contains("Fixing coverage gaps"));
    }

    #[test]
    fn system_prompts_are_nonempty() {
        for kind in [
            ArtifactKind::Contract,
            ArtifactKind::Conformance,
            ArtifactKind::Behavior,
            ArtifactKind::Improve,
        ] {
            assert!(!system_prompt(kind).is_empty());
        }
    }
}
