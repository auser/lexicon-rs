use crate::spec::common::DimensionCategory;
use crate::spec::scoring::ScoreModel;

use super::error::{ScoringError, ScoringResult};

/// The result of evaluating a single dimension.
#[derive(Debug, Clone)]
pub struct DimensionResult {
    /// ID matching a ScoreDimension.
    pub dimension_id: String,
    /// Score value between 0.0 and 1.0.
    pub value: f64,
    /// Whether this dimension passed (value >= threshold or gate passed).
    pub passed: bool,
    /// Human-readable explanation.
    pub explanation: String,
}

/// The result of computing the full score.
#[derive(Debug, Clone)]
pub struct ScoreReport {
    /// Overall weighted score (0.0 to 1.0).
    pub total_score: f64,
    /// Whether all required dimensions passed.
    pub all_required_passed: bool,
    /// Overall verdict.
    pub verdict: Verdict,
    /// Per-dimension results.
    pub dimensions: Vec<DimensionResult>,
}

/// Overall scoring verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
}

/// Compute the score from a model and dimension results.
pub fn compute_score(
    model: &ScoreModel,
    results: &[DimensionResult],
) -> ScoringResult<ScoreReport> {
    if model.dimensions.is_empty() {
        return Err(ScoringError::NoDimensions);
    }

    let total_weight: u32 = model
        .dimensions
        .iter()
        .filter(|d| d.category != DimensionCategory::Advisory)
        .map(|d| d.weight)
        .sum();

    if total_weight == 0 {
        return Err(ScoringError::NoDimensions);
    }

    let mut weighted_sum = 0.0;
    let mut all_required_passed = true;

    for dim in &model.dimensions {
        if dim.category == DimensionCategory::Advisory {
            continue;
        }

        let result = results
            .iter()
            .find(|r| r.dimension_id == dim.id)
            .ok_or_else(|| ScoringError::MissingResult {
                id: dim.id.clone(),
            })?;

        weighted_sum += result.value * dim.weight as f64;

        if dim.category == DimensionCategory::Required && !result.passed {
            all_required_passed = false;
        }
    }

    let total_score = weighted_sum / total_weight as f64;

    let verdict = if !all_required_passed {
        Verdict::Fail
    } else if total_score >= model.thresholds.pass {
        Verdict::Pass
    } else if total_score >= model.thresholds.warn {
        Verdict::Warn
    } else {
        Verdict::Fail
    };

    Ok(ScoreReport {
        total_score,
        all_required_passed,
        verdict,
        dimensions: results.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::scoring::ScoreModel;

    fn make_result(id: &str, value: f64, passed: bool) -> DimensionResult {
        DimensionResult {
            dimension_id: id.to_string(),
            value,
            passed,
            explanation: format!("{id}: {value}"),
        }
    }

    #[test]
    fn test_compute_score_all_pass() {
        let model = ScoreModel::default_model();
        let results: Vec<DimensionResult> = model
            .dimensions
            .iter()
            .map(|d| make_result(&d.id, 1.0, true))
            .collect();

        let report = compute_score(&model, &results).unwrap();
        assert_eq!(report.verdict, Verdict::Pass);
        assert!(report.all_required_passed);
        assert!((report.total_score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_score_required_fails() {
        let model = ScoreModel::default_model();
        let results: Vec<DimensionResult> = model
            .dimensions
            .iter()
            .map(|d| {
                if d.id == "correctness" {
                    make_result(&d.id, 0.0, false)
                } else {
                    make_result(&d.id, 1.0, true)
                }
            })
            .collect();

        let report = compute_score(&model, &results).unwrap();
        assert_eq!(report.verdict, Verdict::Fail);
        assert!(!report.all_required_passed);
    }

    #[test]
    fn test_compute_score_warn_range() {
        let model = ScoreModel::default_model();
        // Give exactly enough to be in warn range (0.6 <= score < 0.8)
        let results: Vec<DimensionResult> = model
            .dimensions
            .iter()
            .map(|d| make_result(&d.id, 0.7, true))
            .collect();

        let report = compute_score(&model, &results).unwrap();
        assert_eq!(report.verdict, Verdict::Warn);
    }

    #[test]
    fn test_empty_model() {
        let model = ScoreModel {
            schema_version: crate::spec::version::SchemaVersion::CURRENT,
            dimensions: vec![],
            thresholds: Default::default(),
        };
        let result = compute_score(&model, &[]);
        assert!(result.is_err());
    }
}
