use serde::{Deserialize, Serialize};

use super::common::DimensionCategory;
use super::version::SchemaVersion;

/// The gates model defines verification checks that must pass.
///
/// Gates are concrete commands that run during `lexicon verify`.
/// Required gates block the build. Scored gates contribute to the
/// overall score. Advisory gates are informational.
///
/// Stored at `specs/gates.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatesModel {
    pub schema_version: SchemaVersion,
    #[serde(default)]
    pub gates: Vec<Gate>,
}

/// A single verification gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    /// Unique identifier, e.g. "fmt", "clippy", "unit-tests".
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Shell command to execute.
    pub command: String,
    /// Whether this gate is required, scored, or advisory.
    #[serde(default)]
    pub category: DimensionCategory,
    /// Timeout in seconds. None means use default (300s).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Whether this gate may be skipped. Required gates default to false.
    #[serde(default)]
    pub allow_skip: bool,
}

impl GatesModel {
    /// Create a default gates model with standard Rust gates.
    pub fn default_model() -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            gates: vec![
                Gate {
                    id: "fmt".to_string(),
                    label: "Format Check".to_string(),
                    command: "cargo fmt -- --check".to_string(),
                    category: DimensionCategory::Required,
                    timeout_secs: Some(60),
                    allow_skip: false,
                },
                Gate {
                    id: "clippy".to_string(),
                    label: "Clippy Lints".to_string(),
                    command: "cargo clippy -- -D warnings".to_string(),
                    category: DimensionCategory::Required,
                    timeout_secs: Some(120),
                    allow_skip: false,
                },
                Gate {
                    id: "unit-tests".to_string(),
                    label: "Unit Tests".to_string(),
                    command: "cargo test".to_string(),
                    category: DimensionCategory::Required,
                    timeout_secs: Some(300),
                    allow_skip: false,
                },
                Gate {
                    id: "doc-tests".to_string(),
                    label: "Documentation Tests".to_string(),
                    command: "cargo test --doc".to_string(),
                    category: DimensionCategory::Scored,
                    timeout_secs: Some(120),
                    allow_skip: true,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gates_model_toml_roundtrip() {
        let model = GatesModel::default_model();
        let toml_str = toml::to_string_pretty(&model).unwrap();
        let parsed: GatesModel = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.gates.len(), 4);
        assert_eq!(parsed.gates[0].id, "fmt");
        assert!(!parsed.gates[0].allow_skip);
    }

    #[test]
    fn test_required_gates_cannot_skip() {
        let model = GatesModel::default_model();
        for gate in &model.gates {
            if gate.category == DimensionCategory::Required {
                assert!(!gate.allow_skip, "Required gate {} must not allow skip", gate.id);
            }
        }
    }
}
