use serde::{Deserialize, Serialize};

use super::common::{DimensionCategory, ScoreSource};
use super::version::SchemaVersion;

/// The scoring model defines how repository health is measured.
///
/// Each dimension contributes a weighted score. Dimensions can be
/// required (must pass), scored (contributes to total), or advisory
/// (informational only).
///
/// Stored at `specs/scoring/model.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreModel {
    pub schema_version: SchemaVersion,
    #[serde(default)]
    pub dimensions: Vec<ScoreDimension>,
    #[serde(default)]
    pub thresholds: ScoreThresholds,
}

/// A single scoring dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDimension {
    /// Unique identifier, e.g. "correctness", "conformance-coverage".
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Weight in the total score calculation.
    pub weight: u32,
    /// Whether this dimension is required, scored, or advisory.
    #[serde(default)]
    pub category: DimensionCategory,
    /// Where the dimension's value comes from.
    pub source: ScoreSource,
}

/// Thresholds for the overall score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreThresholds {
    /// Score at or above this value is "pass" (0.0 - 1.0).
    pub pass: f64,
    /// Score at or above this value but below pass is "warn".
    pub warn: f64,
}

impl Default for ScoreThresholds {
    fn default() -> Self {
        Self {
            pass: 0.8,
            warn: 0.6,
        }
    }
}

impl ScoreModel {
    /// Create a default scoring model with standard dimensions.
    pub fn default_model() -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            dimensions: vec![
                ScoreDimension {
                    id: "correctness".to_string(),
                    label: "Correctness".to_string(),
                    weight: 30,
                    category: DimensionCategory::Required,
                    source: ScoreSource::Gate,
                },
                ScoreDimension {
                    id: "conformance-coverage".to_string(),
                    label: "Conformance Coverage".to_string(),
                    weight: 20,
                    category: DimensionCategory::Scored,
                    source: ScoreSource::TestSuite,
                },
                ScoreDimension {
                    id: "behavior-pass-rate".to_string(),
                    label: "Behavior Pass Rate".to_string(),
                    weight: 10,
                    category: DimensionCategory::Scored,
                    source: ScoreSource::TestSuite,
                },
                ScoreDimension {
                    id: "lint-quality".to_string(),
                    label: "Lint Quality".to_string(),
                    weight: 10,
                    category: DimensionCategory::Scored,
                    source: ScoreSource::Gate,
                },
                ScoreDimension {
                    id: "doc-completeness".to_string(),
                    label: "Documentation Completeness".to_string(),
                    weight: 10,
                    category: DimensionCategory::Advisory,
                    source: ScoreSource::Manual,
                },
                ScoreDimension {
                    id: "panic-safety".to_string(),
                    label: "Panic Safety".to_string(),
                    weight: 5,
                    category: DimensionCategory::Scored,
                    source: ScoreSource::Gate,
                },
                ScoreDimension {
                    id: "contract-coverage".to_string(),
                    label: "Contract Coverage".to_string(),
                    weight: 10,
                    category: DimensionCategory::Scored,
                    source: ScoreSource::Coverage,
                },
                ScoreDimension {
                    id: "api-drift".to_string(),
                    label: "API Drift".to_string(),
                    weight: 5,
                    category: DimensionCategory::Advisory,
                    source: ScoreSource::Gate,
                },
            ],
            thresholds: ScoreThresholds::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_model_toml_roundtrip() {
        let model = ScoreModel::default_model();
        let toml_str = toml::to_string_pretty(&model).unwrap();
        let parsed: ScoreModel = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.dimensions.len(), 8);
        assert_eq!(parsed.thresholds.pass, 0.8);
    }

    #[test]
    fn test_weights_sum() {
        let model = ScoreModel::default_model();
        let total: u32 = model.dimensions.iter().map(|d| d.weight).sum();
        assert_eq!(total, 100);
    }
}
