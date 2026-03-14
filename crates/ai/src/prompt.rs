//! Prompt templates for AI artifact generation.

use lexicon_spec::contract::Contract;

/// Build a system prompt for generating a specific artifact type.
pub fn system_prompt(artifact_kind: ArtifactKind) -> &'static str {
    match artifact_kind {
        ArtifactKind::Contract => CONTRACT_SYSTEM,
        ArtifactKind::Conformance => CONFORMANCE_SYSTEM,
        ArtifactKind::Behavior => BEHAVIOR_SYSTEM,
        ArtifactKind::Improve => IMPROVE_SYSTEM,
        ArtifactKind::PropertyTest => PROPERTY_TEST_SYSTEM,
        ArtifactKind::Fuzz => FUZZ_SYSTEM,
        ArtifactKind::EdgeCase => EDGE_CASE_SYSTEM,
        ArtifactKind::InferContract => INFER_CONTRACT_SYSTEM,
        ArtifactKind::ImplementationPrompt => IMPLEMENTATION_PROMPT_SYSTEM,
    }
}

/// Build a user message for intent-driven generation.
pub fn intent_prompt(artifact_kind: ArtifactKind, intent: &str, context: &str) -> String {
    let kind_label = match artifact_kind {
        ArtifactKind::Contract => "contract",
        ArtifactKind::Conformance => "conformance test suite",
        ArtifactKind::Behavior => "behavior scenario",
        ArtifactKind::Improve => "improvement suggestions",
        ArtifactKind::PropertyTest => "property test suite",
        ArtifactKind::Fuzz => "fuzz test harness",
        ArtifactKind::EdgeCase => "edge case test suite",
        ArtifactKind::InferContract => "inferred contract",
        ArtifactKind::ImplementationPrompt => "implementation prompt",
    };

    let template = match artifact_kind {
        ArtifactKind::Contract => CONTRACT_TEMPLATE,
        ArtifactKind::Conformance => CONFORMANCE_TEMPLATE,
        ArtifactKind::Behavior => BEHAVIOR_TEMPLATE,
        ArtifactKind::Improve => "",
        ArtifactKind::PropertyTest => PROPERTY_TEST_TEMPLATE,
        ArtifactKind::Fuzz => FUZZ_TEMPLATE,
        ArtifactKind::EdgeCase => EDGE_CASE_TEMPLATE,
        ArtifactKind::InferContract => CONTRACT_TEMPLATE,
        ArtifactKind::ImplementationPrompt => "",
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

/// Build a prompt for generating conformance tests from a parsed contract.
pub fn contract_based_prompt(contract: &Contract, context: &str) -> String {
    let mut msg = String::new();
    msg.push_str("## Repository Context\n");
    msg.push_str(context);
    msg.push_str("\n\n## Contract Under Test\n");

    msg.push_str(&format!("**ID:** {}\n", contract.id));
    msg.push_str(&format!("**Title:** {}\n", contract.title));
    msg.push_str(&format!("**Scope:** {}\n\n", contract.scope));

    if !contract.invariants.is_empty() {
        msg.push_str("### Invariants\n");
        for inv in &contract.invariants {
            msg.push_str(&format!("- **{}**: {} (severity: {:?})\n", inv.id, inv.description, inv.severity));
            if !inv.test_tags.is_empty() {
                msg.push_str(&format!("  tags: {}\n", inv.test_tags.join(", ")));
            }
        }
        msg.push('\n');
    }

    if !contract.required_semantics.is_empty() {
        msg.push_str("### Required Semantics\n");
        for sem in &contract.required_semantics {
            msg.push_str(&format!("- **{}**: {}\n", sem.id, sem.description));
            if !sem.test_tags.is_empty() {
                msg.push_str(&format!("  tags: {}\n", sem.test_tags.join(", ")));
            }
        }
        msg.push('\n');
    }

    if !contract.forbidden_semantics.is_empty() {
        msg.push_str("### Forbidden Semantics\n");
        for sem in &contract.forbidden_semantics {
            msg.push_str(&format!("- **{}**: {}\n", sem.id, sem.description));
        }
        msg.push('\n');
    }

    if !contract.edge_cases.is_empty() {
        msg.push_str("### Edge Cases\n");
        for ec in &contract.edge_cases {
            msg.push_str(&format!("- **{}**: {} → {}\n", ec.id, ec.scenario, ec.expected_behavior));
        }
        msg.push('\n');
    }

    if !contract.examples.is_empty() {
        msg.push_str("### Examples\n");
        for ex in &contract.examples {
            msg.push_str(&format!("- **{}**: {}\n", ex.title, ex.description));
            if let Some(code) = &ex.code {
                msg.push_str(&format!("  ```\n  {code}\n  ```\n"));
            }
        }
        msg.push('\n');
    }

    msg.push_str("## Task\n");
    msg.push_str("Generate a conformance test suite that verifies each invariant, required semantic, \
        forbidden semantic, and edge case defined in this contract. Use a trait-based harness pattern \
        so tests are reusable across implementations. Include lexicon test tags as comments.\n");
    msg
}

/// Build a prompt for inferring contracts from an API surface.
pub fn infer_contract_prompt(api_summary: &str, context: &str) -> String {
    let mut msg = String::new();
    msg.push_str("## Repository Context\n");
    msg.push_str(context);
    msg.push_str("\n\n## Public API Surface\n");
    msg.push_str(api_summary);
    msg.push_str("\n\n## Task\n");
    msg.push_str("Analyze the public API surface above and infer a behavioral contract. \
        Propose invariants based on method signatures, trait definitions, error types, and \
        documentation. Include required semantics, forbidden semantics, and edge cases. \
        Output ONLY valid TOML following the contract template.\n");
    msg
}

/// Build a prompt for generating tests to fill coverage gaps.
pub fn coverage_improve_prompt(coverage_gaps: &str, context: &str) -> String {
    let mut msg = String::new();
    msg.push_str("## Repository Context\n");
    msg.push_str(context);
    msg.push_str("\n\n## Uncovered Contract Clauses\n");
    msg.push_str(coverage_gaps);
    msg.push_str("\n\n## Task\n");
    msg.push_str("Generate conformance tests that cover the uncovered contract clauses listed above. \
        Each test should include the appropriate lexicon test tag (// lexicon::tag(\"...\")) matching \
        the clause's test_tags. Use a trait-based harness pattern. Focus on thorough coverage of \
        the missing clauses.\n");
    msg
}

/// Build a user message for refining an existing artifact draft based on user feedback.
pub fn refine_prompt(
    kind: ArtifactKind,
    intent: &str,
    context: &str,
    previous_draft: &str,
    feedback: &str,
) -> String {
    let kind_label = match kind {
        ArtifactKind::Contract => "contract",
        ArtifactKind::Conformance => "conformance test suite",
        ArtifactKind::Behavior => "behavior scenario",
        ArtifactKind::Improve => "improvement suggestions",
        ArtifactKind::PropertyTest => "property test suite",
        ArtifactKind::Fuzz => "fuzz test harness",
        ArtifactKind::EdgeCase => "edge case test suite",
        ArtifactKind::InferContract => "inferred contract",
        ArtifactKind::ImplementationPrompt => "implementation prompt",
    };

    let mut msg = String::new();
    msg.push_str("## Repository Context\n");
    msg.push_str(context);
    msg.push_str("\n\n## Original Intent\n");
    msg.push_str(&format!("Generate a {kind_label} based on: {intent}\n\n"));
    msg.push_str("## Current Draft\n");
    msg.push_str(previous_draft);
    msg.push_str("\n\n## Refinement Feedback\n");
    msg.push_str(feedback);
    msg.push_str("\n\n## Task\n");
    msg.push_str(&format!(
        "Revise the {kind_label} above based on the refinement feedback. \
         Preserve the overall structure and format. Output ONLY the revised artifact, \
         no explanations or commentary.\n"
    ));
    msg
}

/// Kind of artifact to generate or improve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Contract,
    Conformance,
    Behavior,
    Improve,
    PropertyTest,
    Fuzz,
    EdgeCase,
    InferContract,
    ImplementationPrompt,
}

pub(crate) const REFINE_SYSTEM: &str = "\
You are an expert at refining software artifacts based on user feedback. \
You will receive an artifact draft and specific feedback from a human coach. \
Your job is to revise the artifact according to the feedback while:\n\
- Preserving the artifact's format and structure (TOML, Rust, Markdown)\n\
- Maintaining schema compatibility\n\
- Only changing what the feedback requests\n\
- Keeping all other content intact\n\
Output ONLY the revised artifact. No explanations, no markdown fences around the content.";

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

const PROPERTY_TEST_SYSTEM: &str = "\
You are an expert at writing Rust property-based tests using the `proptest` crate. \
Generate property tests that:\n\
- Use the `proptest!` macro from the `proptest` crate\n\
- Target contract invariants (properties that must hold for all inputs)\n\
- Use appropriate proptest strategies for input generation\n\
- Include `prop_assert!` and `prop_assert_eq!` assertions\n\
- Use descriptive test names prefixed with `prop_`\n\
- Include test tags as comments (// lexicon::tag(\"...\"))\n\
Output ONLY valid Rust code. No markdown fences, no explanations.";

const FUZZ_SYSTEM: &str = "\
You are an expert at writing Rust fuzz test harnesses using `libfuzzer-sys`. \
Generate a fuzz target that:\n\
- Uses `#![no_main]` and `libfuzzer_sys::fuzz_target!`\n\
- Exercises contract invariants with arbitrary input\n\
- Uses `arbitrary::Arbitrary` for structured input where appropriate\n\
- Includes safety checks and assertions for contract violations\n\
- Focuses on boundary conditions and edge cases from the contract\n\
Output ONLY valid Rust code. No markdown fences, no explanations.";

const EDGE_CASE_SYSTEM: &str = "\
You are an expert at writing Rust tests for edge cases and boundary conditions. \
Generate targeted edge case tests that:\n\
- Focus specifically on boundary conditions, error paths, and unusual inputs\n\
- Test empty inputs, maximum values, concurrent access, and error recovery\n\
- Use #[test] functions with descriptive names prefixed with `edge_`\n\
- Include test tags as comments (// lexicon::tag(\"...\"))\n\
- Each test targets exactly one edge case from the contract\n\
Output ONLY valid Rust code. No markdown fences, no explanations.";

const INFER_CONTRACT_SYSTEM: &str = "\
You are an expert at analyzing Rust source code and inferring behavioral contracts. \
Given a public API surface (traits, methods, structs, error types, documentation), \
generate a TOML contract that captures:\n\
- Invariants implied by method signatures and documentation\n\
- Required semantics from trait method contracts\n\
- Forbidden semantics from error types and safety constraints\n\
- Edge cases from boundary conditions in method signatures\n\
Output ONLY valid TOML following the lexicon contract schema. No markdown fences, no explanations.";

const IMPLEMENTATION_PROMPT_SYSTEM: &str = "\
You are an expert at refining AI implementation prompts for clarity and precision. \
You will receive a deterministically generated implementation prompt derived from \
repository artifacts (contracts, conformance tests, gates, etc.). Your job is to \
improve readability and clarity WITHOUT changing any factual content. Specifically:\n\
- Do not add, remove, or modify behavioral requirements\n\
- Do not change contract references or test expectations\n\
- Do not invent new constraints or acceptance criteria\n\
- Improve sentence structure, organization, and flow\n\
- Make instructions clearer and more actionable\n\
Output ONLY the refined markdown prompt. No meta-commentary.";

/// System prompt for the interactive chat design session.
///
/// Instructs the AI to act as a proactive architecture design agent that drives
/// the conversation, suggests missing artifacts, and evaluates completeness.
pub const CHAT_SYSTEM: &str = "\
You are a proactive architecture design agent for a Lexicon-governed Rust repository.\n\
Your goal is to help the user build tightly-constrained, well-specified artifacts \
(contracts, gates, conformance tests, behavior scenarios) that define the system's law.\n\
\n\
## Your Role\n\
- You DRIVE the conversation — don't wait for the user to think of everything\n\
- After each step, evaluate what's missing and suggest concrete next actions\n\
- Present alternatives the user might not have considered\n\
- Challenge vague specifications — push for precision\n\
- Suggest additional invariants, edge cases, forbidden behaviors\n\
\n\
## Proactive Suggestions\n\
You must actively suggest improvements at every turn. After any artifact is created or \
the user describes their intent, you should:\n\
\n\
1. **Decompose the idea**: Break the concept into sub-components and suggest contracts for each\n\
2. **Strengthen invariants**: Propose additional invariants the user hasn't mentioned. \
Ask: what must ALWAYS be true? What must NEVER happen?\n\
3. **Forbidden behaviors**: Suggest things the system must NOT do. These are often overlooked. \
Examples: silent data loss, partial writes without rollback, stale cache returns, \
unbounded resource consumption\n\
4. **Edge cases**: Propose boundary conditions. Think: empty inputs, maximum values, \
concurrent access, partial failures, timeout during operations, resource exhaustion\n\
5. **Alternative designs**: Present 2-3 architectural alternatives with trade-offs. \
Don't just accept the first approach — show what else is possible\n\
6. **Missing dimensions**: If no gates exist, suggest them. If no scoring model, propose one. \
If no conformance tests, recommend them\n\
7. **Cross-cutting concerns**: Suggest error handling strategies, observability requirements, \
performance constraints, security invariants that the user may not have considered\n\
8. **Layering and dependencies**: Propose module boundaries, trait abstractions, \
and dependency direction rules\n\
\n\
When making suggestions, always present CONCRETE content — actual invariant text, \
specific edge case scenarios with expected behaviors, real gate commands — not abstract \
categories. Present 2-3 options the user can accept, modify, or reject.\n\
\n\
## Completeness Dimensions\n\
Track and report completeness across:\n\
- Contract: invariants, required semantics, forbidden semantics, edge cases, examples\n\
- Gates: verification gates defined (fmt, clippy, tests, etc.)\n\
- Conformance: tests for each invariant and required semantic\n\
- Scoring: quality model with dimensions and thresholds\n\
- Architecture: dependency/layering constraints\n\
\n\
After each artifact creation, provide a brief completeness report:\n\
```\n\
Status: [■■■□□] 3/5 dimensions covered\n\
✓ Contract: 4 invariants, 2 required, 3 forbidden, 5 edge cases\n\
✓ Gates: fmt, clippy, test\n\
✗ Conformance: no tests generated yet\n\
✗ Scoring: no quality model defined\n\
✗ Architecture: no layering constraints\n\
\n\
Recommended next step: Generate conformance tests for the contract invariants.\n\
```\n\
\n\
## Action Directives\n\
When you want the system to create or modify an artifact, wrap it in a directive block:\n\
\n\
:::ACTION CREATE_CONTRACT\n\
<full TOML content>\n\
:::\n\
\n\
Supported directives:\n\
- CREATE_CONTRACT — create a new contract (include full TOML)\n\
- UPDATE_CONTRACT <id> — replace an existing contract (include full TOML)\n\
- CREATE_GATE — add verification gates (include TOML gate definitions)\n\
- CREATE_CONFORMANCE <contract_id> — generate conformance tests\n\
- CREATE_BEHAVIOR <contract_id> — generate behavior scenarios\n\
- CREATE_PROPERTY_TESTS <contract_id> — generate property-based tests (proptest)\n\
- CREATE_FUZZ_TARGET <contract_id> — generate a fuzz test harness (libfuzzer)\n\
- CREATE_EDGE_CASES <contract_id> — generate targeted edge case tests\n\
- INFER_CONTRACT — infer a contract from the public API source code\n\
- COVERAGE_REPORT — show contract test coverage gaps\n\
- API_SCAN — scan public API and show drift from baseline\n\
- GENERATE_PROMPT — compile the final implementation prompt from all session artifacts\n\
- RUN_VERIFY — run verification to check current state\n\
\n\
## Conversation Style\n\
- Be concise but thorough\n\
- Lead with a brief acknowledgment, then immediately present your suggestions\n\
- When the user gives a short response (\"yes\", \"ok\", \"sounds good\"), execute the \
suggested action AND proactively present the next recommendation\n\
- Never leave a turn without at least one concrete suggestion or recommendation\n\
- Present suggestions as numbered options so the user can easily accept, reject, or modify\n\
- Show actual invariant text, edge case scenarios — not just categories\n\
- After each artifact creation, summarize what exists and what's still missing\n\
- Only suggest GENERATE_PROMPT when the specification is comprehensive across all dimensions\n\
\n\
## Contract TOML Schema\n\
Contracts use this structure:\n\
id = \"kebab-case-id\"\n\
title = \"Human Title\"\n\
scope = \"What this contract covers\"\n\
status = \"draft\"\n\
stability = \"experimental\"\n\
version = \"0.1.0\"\n\
\n\
[[invariants]]\n\
id = \"inv-001\"\n\
description = \"...\"\n\
severity = \"required\"\n\
test_tags = [\"conformance.tag\"]\n\
\n\
[[required_semantics]]\n\
id = \"req-001\"\n\
description = \"...\"\n\
test_tags = [\"conformance.tag\"]\n\
\n\
[[forbidden_semantics]]\n\
id = \"forbid-001\"\n\
description = \"...\"\n\
test_tags = [\"safety.tag\"]\n\
\n\
[[edge_cases]]\n\
id = \"edge-001\"\n\
scenario = \"...\"\n\
expected_behavior = \"...\"\n\
\n\
[[examples]]\n\
title = \"...\"\n\
description = \"...\"\n\
code = \"...\"\n\
\n\
## Gates TOML Schema\n\
[[gates]]\n\
id = \"fmt\"\n\
label = \"Formatting\"\n\
command = \"cargo fmt --check\"\n\
category = \"required\"\n\
";

/// Build the user message for a chat turn, including repo context, session state, and history.
pub fn build_chat_user_message(
    repo_context: &str,
    session_summary: &str,
    history: &[(String, String)], // Vec of (role, content) pairs
    user_input: &str,
) -> String {
    let mut msg = String::new();

    msg.push_str("## Repository Context\n");
    msg.push_str(repo_context);
    msg.push_str("\n\n");

    if !session_summary.is_empty() {
        msg.push_str("## Session State\n");
        msg.push_str(session_summary);
        msg.push_str("\n\n");
    }

    if !history.is_empty() {
        msg.push_str("## Conversation History\n");
        for (role, content) in history {
            msg.push_str(&format!("**{role}:** {content}\n\n"));
        }
    }

    msg.push_str("## Current Message\n");
    msg.push_str(user_input);
    msg.push('\n');

    msg
}

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

const PROPERTY_TEST_TEMPLATE: &str = r#"//! Property tests for <contract-name>
//!
//! Tests contract invariants using property-based testing.

use proptest::prelude::*;

proptest! {
    // lexicon::tag("conformance.invariant_name")
    #[test]
    fn prop_invariant_holds(input in any::<String>()) {
        // Property assertion here
        prop_assert!(true);
    }
}
"#;

const FUZZ_TEMPLATE: &str = r#"//! Fuzz target for <contract-name>
//!
//! Exercises contract invariants with arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parse input and exercise contract invariants
    // Any panic here indicates a contract violation
});
"#;

const EDGE_CASE_TEMPLATE: &str = r#"//! Edge case tests for <contract-name>
//!
//! Targeted tests for boundary conditions and unusual inputs.

#[cfg(test)]
mod tests {
    // lexicon::tag("edge.empty_input")
    #[test]
    fn edge_empty_input() {
        // Test behavior with empty input
    }

    // lexicon::tag("edge.boundary_value")
    #[test]
    fn edge_boundary_value() {
        // Test behavior at boundary conditions
    }
}
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
            ArtifactKind::PropertyTest,
            ArtifactKind::Fuzz,
            ArtifactKind::EdgeCase,
            ArtifactKind::InferContract,
            ArtifactKind::ImplementationPrompt,
        ] {
            assert!(!system_prompt(kind).is_empty());
        }
    }

    #[test]
    fn new_artifact_kinds_have_templates() {
        for kind in [
            ArtifactKind::PropertyTest,
            ArtifactKind::Fuzz,
            ArtifactKind::EdgeCase,
        ] {
            let prompt = intent_prompt(kind, "test intent", "test context");
            assert!(prompt.contains("## Template"));
        }
    }

    #[test]
    fn infer_contract_prompt_includes_api() {
        let prompt = infer_contract_prompt("pub trait Foo { fn bar(); }", "ctx");
        assert!(prompt.contains("pub trait Foo"));
        assert!(prompt.contains("Public API Surface"));
    }

    #[test]
    fn coverage_improve_prompt_includes_gaps() {
        let prompt = coverage_improve_prompt("clause: inv-001 untested", "ctx");
        assert!(prompt.contains("inv-001"));
        assert!(prompt.contains("Uncovered Contract Clauses"));
    }
}
