use serde::{Deserialize, Serialize};

/// Status of a contract in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    /// Contract is being drafted, not yet enforced.
    Draft,
    /// Contract is active and enforced.
    Active,
    /// Contract is deprecated, still enforced but expected to be replaced.
    Deprecated,
    /// Contract is retired, no longer enforced.
    Retired,
}

impl Default for ContractStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Stability level of a contract or artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stability {
    /// Experimental: may change or be removed at any time.
    Experimental,
    /// Unstable: expected to stabilize but may still change.
    Unstable,
    /// Stable: changes require versioned migration.
    Stable,
    /// Frozen: must not change without extraordinary justification.
    Frozen,
}

impl Default for Stability {
    fn default() -> Self {
        Self::Experimental
    }
}

/// Severity level for invariants and checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Must pass. Failure blocks verification.
    Required,
    /// Should pass. Contributes to score but does not block.
    Advisory,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Required
    }
}

/// Type of repository being managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoType {
    Library,
    Binary,
    Workspace,
}

impl Default for RepoType {
    fn default() -> Self {
        Self::Library
    }
}

/// Naming convention preference for generated artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingConvention {
    SnakeCase,
    KebabCase,
}

impl Default for NamingConvention {
    fn default() -> Self {
        Self::KebabCase
    }
}

/// Style of conformance testing to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceStyle {
    /// Uses trait-based test harnesses.
    TraitBased,
    /// Uses factory functions to create test instances.
    FactoryBased,
}

impl Default for ConformanceStyle {
    fn default() -> Self {
        Self::TraitBased
    }
}

/// Category for scoring dimensions and gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionCategory {
    /// Must pass. Failure blocks the overall gate.
    Required,
    /// Contributes to the numeric score.
    Scored,
    /// Informational only. Does not affect pass/fail.
    Advisory,
}

impl Default for DimensionCategory {
    fn default() -> Self {
        Self::Scored
    }
}

/// Source of a score dimension value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreSource {
    Gate,
    TestSuite,
    Coverage,
    Manual,
}

/// Actor that performed an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    User,
    Ai,
    System,
}

/// Kind of conversation workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKind {
    Init,
    ContractNew,
    ContractEdit,
    ConformanceAdd,
    BehaviorAdd,
    ScoreInit,
    GateInit,
    Improve,
}

/// Status of a conversation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Completed,
    Abandoned,
}

/// Type of step in a conversation workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Question,
    UserInput,
    Proposal,
    Refinement,
    Validation,
    Write,
    Info,
}

/// Type of auditable action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    RepoInit,
    ContractCreate,
    ContractUpdate,
    ContractStatusChange,
    ConformanceCreate,
    ConformanceUpdate,
    BehaviorCreate,
    BehaviorUpdate,
    ScoreModelChange,
    GateModelChange,
    GateWeakeningAttempt,
    AiImprove,
    AiImproveRejected,
    VerifyRun,
    ClaudeSyncRun,
    TestDeletion,
    ThresholdChange,
    ApiScan,
    ApiDiff,
    CoverageReport,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_status_serde() {
        let s = ContractStatus::Active;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"active\"");
        let parsed: ContractStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn test_stability_ordering() {
        assert!(Stability::Experimental < Stability::Unstable);
        assert!(Stability::Unstable < Stability::Stable);
        assert!(Stability::Stable < Stability::Frozen);
    }

    #[test]
    fn test_all_enums_roundtrip_json() {
        macro_rules! roundtrip {
            ($t:ty, $val:expr) => {
                let json = serde_json::to_string(&$val).unwrap();
                let parsed: $t = serde_json::from_str(&json).unwrap();
                assert_eq!($val, parsed);
            };
        }

        roundtrip!(ContractStatus, ContractStatus::Draft);
        roundtrip!(Stability, Stability::Frozen);
        roundtrip!(Severity, Severity::Advisory);
        roundtrip!(RepoType, RepoType::Workspace);
        roundtrip!(NamingConvention, NamingConvention::SnakeCase);
        roundtrip!(ConformanceStyle, ConformanceStyle::FactoryBased);
        roundtrip!(DimensionCategory, DimensionCategory::Required);
        roundtrip!(ScoreSource, ScoreSource::Gate);
        roundtrip!(Actor, Actor::Ai);
        roundtrip!(WorkflowKind, WorkflowKind::ContractNew);
        roundtrip!(SessionStatus, SessionStatus::Abandoned);
        roundtrip!(StepType, StepType::Proposal);
        roundtrip!(AuditAction, AuditAction::AiImprove);
    }
}
