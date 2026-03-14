use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::{ContractStatus, Severity, Stability};
use crate::version::SchemaVersion;

/// A contract defines the stable behavioral specification for a component.
///
/// Contracts are the primary artifact in lexicon. They declare what a
/// component must do, what it must not do, and what is explicitly out of scope.
///
/// Stored at `specs/contracts/<id>.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub schema_version: SchemaVersion,
    /// Unique slug identifier, e.g. "key-value-store".
    pub id: String,
    pub title: String,
    /// A longer description of the contract's purpose and context.
    #[serde(default)]
    pub description: String,
    pub status: ContractStatus,
    pub stability: Stability,
    /// High-level scope description.
    pub scope: String,
    /// What the component provides.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Invariants that must always hold.
    #[serde(default)]
    pub invariants: Vec<Invariant>,
    /// Behavior that is required.
    #[serde(default)]
    pub required_semantics: Vec<Semantic>,
    /// Behavior that is explicitly forbidden.
    #[serde(default)]
    pub forbidden_semantics: Vec<Semantic>,
    /// Edge cases with expected behavior.
    #[serde(default)]
    pub edge_cases: Vec<EdgeCase>,
    /// Usage examples.
    #[serde(default)]
    pub examples: Vec<Example>,
    /// What this component explicitly does not do.
    #[serde(default)]
    pub non_goals: Vec<String>,
    /// Implementation-level notes (not part of the contract).
    #[serde(default)]
    pub implementation_notes: Vec<String>,
    /// What tests are expected for this contract.
    #[serde(default)]
    pub test_expectations: Vec<String>,
    /// Expected public API items (traits, methods, types) this contract covers.
    #[serde(default)]
    pub expected_api: Vec<String>,
    /// Version history of contract changes.
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An invariant that must always hold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    /// Unique identifier within the contract.
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub severity: Severity,
    /// Tags for linking to conformance tests.
    #[serde(default)]
    pub test_tags: Vec<String>,
}

/// A semantic requirement or prohibition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semantic {
    /// Unique identifier within the contract.
    pub id: String,
    pub description: String,
    /// Tags for linking to conformance tests.
    #[serde(default)]
    pub test_tags: Vec<String>,
}

/// An edge case with expected behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCase {
    pub id: String,
    pub scenario: String,
    pub expected_behavior: String,
}

/// A usage example.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub code: Option<String>,
}

/// A record of a contract change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub version: String,
    pub date: DateTime<Utc>,
    pub description: String,
    pub author: String,
}

impl Contract {
    /// Create a new draft contract with the given id and title.
    pub fn new_draft(id: String, title: String, scope: String) -> Self {
        let now = Utc::now();
        Self {
            schema_version: SchemaVersion::CURRENT,
            id,
            title,
            description: String::new(),
            status: ContractStatus::Draft,
            stability: Stability::Experimental,
            scope,
            capabilities: Vec::new(),
            invariants: Vec::new(),
            required_semantics: Vec::new(),
            forbidden_semantics: Vec::new(),
            edge_cases: Vec::new(),
            examples: Vec::new(),
            non_goals: Vec::new(),
            implementation_notes: Vec::new(),
            test_expectations: Vec::new(),
            expected_api: Vec::new(),
            history: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_toml_roundtrip() {
        let mut contract = Contract::new_draft(
            "key-value-store".to_string(),
            "Key-Value Store Contract".to_string(),
            "Defines the behavior of a basic key-value store".to_string(),
        );
        contract.capabilities.push("get/set/delete operations".to_string());
        contract.invariants.push(Invariant {
            id: "inv-001".to_string(),
            description: "A key set with a value must return that value on get".to_string(),
            severity: Severity::Required,
            test_tags: vec!["conformance".to_string()],
        });
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "get(key) returns None for missing keys".to_string(),
            test_tags: vec!["conformance".to_string(), "basic".to_string()],
        });
        contract.forbidden_semantics.push(Semantic {
            id: "forbid-001".to_string(),
            description: "Must not panic on missing key lookup".to_string(),
            test_tags: vec!["safety".to_string()],
        });

        let toml_str = toml::to_string_pretty(&contract).unwrap();
        let parsed: Contract = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.id, "key-value-store");
        assert_eq!(parsed.invariants.len(), 1);
        assert_eq!(parsed.required_semantics.len(), 1);
        assert_eq!(parsed.forbidden_semantics.len(), 1);
    }

    #[test]
    fn test_contract_defaults() {
        let c = Contract::new_draft(
            "test".to_string(),
            "Test".to_string(),
            "scope".to_string(),
        );
        assert_eq!(c.status, ContractStatus::Draft);
        assert_eq!(c.stability, Stability::Experimental);
        assert!(c.invariants.is_empty());
    }
}
