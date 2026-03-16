use serde::{Deserialize, Serialize};

use super::common::ConformanceStyle;
use super::version::SchemaVersion;

/// Definition of a conformance test suite tied to a contract.
///
/// Conformance suites define reusable test harnesses that verify
/// implementations against a contract's invariants and semantics.
///
/// Stored at `specs/conformance/<id>.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceSuite {
    pub schema_version: SchemaVersion,
    /// Unique identifier for this suite.
    pub id: String,
    /// The contract this suite verifies.
    pub contract_id: String,
    /// Style of conformance harness.
    pub style: ConformanceStyle,
    /// Module path for the generated harness, e.g. "tests::conformance::store".
    pub harness_module: String,
    /// Tests that must pass for conformance.
    #[serde(default)]
    pub required_tests: Vec<ConformanceTest>,
    /// Optional tests that improve coverage but don't block.
    #[serde(default)]
    pub optional_tests: Vec<ConformanceTest>,
    /// Shared fixtures referenced by tests.
    #[serde(default)]
    pub fixtures: Vec<FixtureRef>,
}

/// A single conformance test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceTest {
    /// Unique identifier within the suite.
    pub id: String,
    pub description: String,
    /// Tags for categorization and filtering.
    #[serde(default)]
    pub tags: Vec<String>,
    /// References to contract clause IDs this test verifies.
    #[serde(default)]
    pub clause_refs: Vec<String>,
}

/// Reference to a shared test fixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureRef {
    pub id: String,
    pub description: String,
    /// Module path where the fixture is defined.
    pub module_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conformance_suite_roundtrip() {
        let suite = ConformanceSuite {
            schema_version: SchemaVersion::CURRENT,
            id: "kv-store-conformance".to_string(),
            contract_id: "key-value-store".to_string(),
            style: ConformanceStyle::TraitBased,
            harness_module: "tests::conformance::kv_store".to_string(),
            required_tests: vec![ConformanceTest {
                id: "test-get-set".to_string(),
                description: "Setting a key then getting it returns the value".to_string(),
                tags: vec!["basic".to_string()],
                clause_refs: vec!["inv-001".to_string()],
            }],
            optional_tests: vec![],
            fixtures: vec![],
        };
        let toml_str = toml::to_string_pretty(&suite).unwrap();
        let parsed: ConformanceSuite = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.contract_id, "key-value-store");
        assert_eq!(parsed.required_tests.len(), 1);
    }
}
