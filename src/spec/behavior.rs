use serde::{Deserialize, Serialize};

use super::version::SchemaVersion;

/// A BDD-style behavior scenario for readable intent documentation.
///
/// Behavior scenarios complement contracts by providing narrative
/// descriptions of expected behavior, useful for acceptance testing
/// and communicating intent to stakeholders.
///
/// Stored at `specs/behavior/<id>.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorScenario {
    pub schema_version: SchemaVersion,
    /// Unique identifier for this scenario.
    pub id: String,
    pub title: String,
    /// Optional contract this scenario relates to.
    #[serde(default)]
    pub contract_id: Option<String>,
    /// Preconditions (Given).
    #[serde(default)]
    pub given: Vec<String>,
    /// Actions (When).
    #[serde(default)]
    pub when: Vec<String>,
    /// Expected outcomes (Then).
    #[serde(default)]
    pub then: Vec<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_scenario_roundtrip() {
        let scenario = BehaviorScenario {
            schema_version: SchemaVersion::CURRENT,
            id: "store-retrieval".to_string(),
            title: "Stored values can be retrieved".to_string(),
            contract_id: Some("key-value-store".to_string()),
            given: vec!["an empty store".to_string()],
            when: vec!["a value is stored with key \"foo\"".to_string()],
            then: vec!["getting key \"foo\" returns the stored value".to_string()],
            tags: vec!["basic".to_string(), "happy-path".to_string()],
        };
        let toml_str = toml::to_string_pretty(&scenario).unwrap();
        let parsed: BehaviorScenario = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.id, "store-retrieval");
        assert_eq!(parsed.given.len(), 1);
    }
}
