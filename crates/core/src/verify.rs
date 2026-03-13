use lexicon_api::diff::ApiDiff;
use lexicon_audit::writer::write_audit_record;
use lexicon_coverage::report::CoverageReport;
use lexicon_gates::result::GateOutcome;
use lexicon_gates::runner::run_all_gates;
use lexicon_repo::layout::RepoLayout;
use lexicon_scoring::engine::{compute_score, DimensionResult, ScoreReport};
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::common::{Actor, AuditAction, DimensionCategory};
use lexicon_spec::contract::Contract;
use lexicon_spec::gates::GatesModel;
use lexicon_spec::scoring::ScoreModel;

use crate::error::CoreResult;

/// Result of a full verification run.
#[derive(Debug)]
pub struct VerifyResult {
    pub gate_results: Vec<lexicon_gates::result::GateResult>,
    pub score_report: Option<ScoreReport>,
    pub coverage_report: Option<CoverageReport>,
    pub api_diff: Option<ApiDiff>,
}

/// Run all gates, compute score, check coverage, and check API drift.
pub fn verify(layout: &RepoLayout) -> CoreResult<VerifyResult> {
    // Load gates model
    let gates_model = lexicon_scaffold::gates::load_gates_model(layout)?
        .unwrap_or_else(GatesModel::default_model);

    // Load score model
    let score_model = lexicon_scaffold::scoring::load_score_model(layout)?;

    // Run gates
    let gate_results = run_all_gates(&gates_model.gates, &layout.root, &[])?;

    // Load contracts for coverage analysis
    let contracts = load_contracts(layout);

    // Coverage analysis
    let coverage_report = if !contracts.is_empty() {
        crate::coverage::coverage_report(layout, &contracts).ok()
    } else {
        None
    };

    // API drift check
    let api_diff = if crate::api::has_baseline(layout) {
        // Scan current API first
        let _ = crate::api::api_scan(layout);
        crate::api::api_diff(layout).ok()
    } else {
        None
    };

    // Compute score if model exists
    let score_report = if let Some(ref model) = score_model {
        let dim_results =
            map_gate_results_to_dimensions(model, &gate_results, &coverage_report, &api_diff);
        compute_score(model, &dim_results).ok()
    } else {
        None
    };

    // Write audit record
    let gates_passed = gate_results
        .iter()
        .all(|r| r.outcome == GateOutcome::Pass || r.outcome == GateOutcome::Skip);
    let total_score = score_report.as_ref().map(|r| r.total_score);
    let mut record = AuditRecord::new(
        AuditAction::VerifyRun,
        Actor::System,
        format!(
            "Verify: {} gates, {} passed",
            gate_results.len(),
            gate_results.iter().filter(|r| r.passed()).count()
        ),
    );
    record.gates_passed = gates_passed;
    record.score_after = total_score;
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(VerifyResult {
        gate_results,
        score_report,
        coverage_report,
        api_diff,
    })
}

/// Load all contracts from the contracts directory.
fn load_contracts(layout: &RepoLayout) -> Vec<Contract> {
    let dir = layout.contracts_dir();
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut contracts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(contract) = toml::from_str::<Contract>(&content) {
                        contracts.push(contract);
                    }
                }
            }
        }
    }
    contracts
}

/// Map gate results, coverage, and API drift to scoring dimension results.
fn map_gate_results_to_dimensions(
    model: &ScoreModel,
    gate_results: &[lexicon_gates::result::GateResult],
    coverage_report: &Option<CoverageReport>,
    api_diff: &Option<ApiDiff>,
) -> Vec<DimensionResult> {
    model
        .dimensions
        .iter()
        .map(|dim| {
            // Check special dimensions first
            if dim.id == "contract-coverage" {
                return coverage_dimension(dim, coverage_report);
            }
            if dim.id == "api-drift" {
                return api_drift_dimension(dim, api_diff);
            }

            // Try to find a matching gate result
            let gate_result = gate_results.iter().find(|r| r.gate_id == dim.id);
            let (value, passed) = if let Some(result) = gate_result {
                let v = if result.passed() { 1.0 } else { 0.0 };
                (v, result.passed())
            } else {
                // No gate matches this dimension — score as advisory pass
                if dim.category == DimensionCategory::Advisory {
                    (1.0, true)
                } else {
                    (0.5, true) // Partial credit for uncovered dimensions
                }
            };

            DimensionResult {
                dimension_id: dim.id.clone(),
                value,
                passed,
                explanation: gate_result
                    .map(|r| {
                        if r.passed() {
                            format!("{}: passed in {}ms", r.gate_id, r.duration_ms)
                        } else {
                            format!("{}: FAILED", r.gate_id)
                        }
                    })
                    .unwrap_or_else(|| format!("{}: no gate configured", dim.id)),
            }
        })
        .collect()
}

fn coverage_dimension(
    dim: &lexicon_spec::scoring::ScoreDimension,
    report: &Option<CoverageReport>,
) -> DimensionResult {
    match report {
        Some(r) => {
            let value = r.overall_coverage_pct / 100.0;
            DimensionResult {
                dimension_id: dim.id.clone(),
                value,
                passed: value >= 0.5,
                explanation: format!(
                    "contract-coverage: {:.1}% ({}/{} clauses)",
                    r.overall_coverage_pct, r.total_covered, r.total_clauses
                ),
            }
        }
        None => DimensionResult {
            dimension_id: dim.id.clone(),
            value: 0.0,
            passed: true,
            explanation: "contract-coverage: no contracts to analyze".to_string(),
        },
    }
}

fn api_drift_dimension(
    dim: &lexicon_spec::scoring::ScoreDimension,
    diff: &Option<ApiDiff>,
) -> DimensionResult {
    match diff {
        Some(d) => {
            let has_breaking = d.has_breaking();
            let value = if has_breaking { 0.0 } else { 1.0 };
            DimensionResult {
                dimension_id: dim.id.clone(),
                value,
                passed: !has_breaking,
                explanation: if d.is_empty() {
                    "api-drift: no changes from baseline".to_string()
                } else {
                    format!("api-drift: {}", d.summary())
                },
            }
        }
        None => DimensionResult {
            dimension_id: dim.id.clone(),
            value: 1.0,
            passed: true,
            explanation: "api-drift: no baseline configured".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_verify_without_gates_or_scoring() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        // Create minimal structure
        crate::init::init_repo_noninteractive(
            &layout,
            "test".to_string(),
            "test".to_string(),
            lexicon_spec::common::RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();

        let result = verify(&layout).unwrap();
        // Default gates will likely fail since there's no Cargo project, but verify runs
        assert!(!result.gate_results.is_empty());
    }
}
