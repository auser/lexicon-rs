//! Mode-aware conversation hints.
//!
//! Provides additional questions and context appropriate to the current
//! operating mode, so workflows only ask relevant questions.

use crate::spec::mode::OperatingMode;

/// Additional questions to ask during artifact creation based on operating mode.
pub fn additional_contract_questions(mode: OperatingMode) -> Vec<ModeHint> {
    let mut hints = vec![
        // Repo mode: always ask these
        ModeHint {
            prompt: "What is the public API surface this contract governs?".into(),
            explanation: "Link contract to specific types, traits, or functions".into(),
            mode: OperatingMode::Repo,
        },
        ModeHint {
            prompt: "What test tags should verify this contract?".into(),
            explanation: "Map contract clauses to conformance tests".into(),
            mode: OperatingMode::Repo,
        },
    ];

    if mode >= OperatingMode::Workspace {
        hints.extend([
            ModeHint {
                prompt: "Which crate(s) does this contract apply to?".into(),
                explanation: "Scope the contract to specific workspace members".into(),
                mode: OperatingMode::Workspace,
            },
            ModeHint {
                prompt: "Does this contract define a shared interface between crates?".into(),
                explanation: "Shared contracts can be validated across workspace members".into(),
                mode: OperatingMode::Workspace,
            },
            ModeHint {
                prompt: "What dependency direction does this contract enforce?".into(),
                explanation: "Architecture rules govern which crates may depend on which".into(),
                mode: OperatingMode::Workspace,
            },
        ]);
    }

    if mode >= OperatingMode::Ecosystem {
        hints.extend([
            ModeHint {
                prompt: "Does this contract apply across multiple repositories?".into(),
                explanation: "Federated contracts are shared across the ecosystem".into(),
                mode: OperatingMode::Ecosystem,
            },
            ModeHint {
                prompt: "Which repos are responsible for implementing this contract?".into(),
                explanation: "Cross-repo ownership must be explicit".into(),
                mode: OperatingMode::Ecosystem,
            },
        ]);
    }

    hints
}

/// A mode-specific conversation hint.
#[derive(Debug, Clone)]
pub struct ModeHint {
    /// The question or prompt text.
    pub prompt: String,
    /// Why this question is relevant.
    pub explanation: String,
    /// The minimum mode level where this hint applies.
    pub mode: OperatingMode,
}

/// Get a mode-appropriate description for the init flow.
pub fn init_mode_description(mode: OperatingMode) -> &'static str {
    match mode {
        OperatingMode::Repo => {
            "Repo Mode: focused on local contract-driven verification.\n\
             Includes contracts, conformance, scoring, gates, API scanning, and coverage."
        }
        OperatingMode::Workspace => {
            "Workspace Mode: extends Repo Mode with architecture governance.\n\
             Adds crate roles, dependency law, shared contracts, and architecture graphs."
        }
        OperatingMode::Ecosystem => {
            "Ecosystem Mode: full governance across multiple repositories.\n\
             Adds repo roles, federated contracts, cross-repo verification, and impact analysis."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_mode_has_base_questions() {
        let hints = additional_contract_questions(OperatingMode::Repo);
        assert_eq!(hints.len(), 2);
    }

    #[test]
    fn workspace_mode_has_more_questions() {
        let hints = additional_contract_questions(OperatingMode::Workspace);
        assert!(hints.len() > 2);
        assert!(hints.iter().any(|h| h.prompt.contains("crate")));
    }

    #[test]
    fn ecosystem_mode_has_most_questions() {
        let hints = additional_contract_questions(OperatingMode::Ecosystem);
        assert!(hints.len() > 5);
        assert!(hints.iter().any(|h| h.prompt.contains("repositories")));
    }

    #[test]
    fn mode_descriptions_are_nonempty() {
        for mode in [OperatingMode::Repo, OperatingMode::Workspace, OperatingMode::Ecosystem] {
            assert!(!init_mode_description(mode).is_empty());
        }
    }
}
