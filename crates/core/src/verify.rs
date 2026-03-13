use lexicon_audit::writer::write_audit_record;
use lexicon_gates::result::GateOutcome;
use lexicon_gates::runner::run_all_gates;
use lexicon_repo::layout::RepoLayout;
use lexicon_scoring::engine::{compute_score, DimensionResult, ScoreReport};
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::common::{Actor, AuditAction, DimensionCategory};
use lexicon_spec::gates::GatesModel;
use lexicon_spec::scoring::ScoreModel;

use crate::error::CoreResult;

/// Result of a full verification run.
#[derive(Debug)]
pub struct VerifyResult {
    pub gate_results: Vec<lexicon_gates::result::GateResult>,
    pub score_report: Option<ScoreReport>,
}

/// Run all gates and compute the score.
pub fn verify(layout: &RepoLayout) -> CoreResult<VerifyResult> {
    // Load gates model
    let gates_model = lexicon_scaffold::gates::load_gates_model(layout)?
        .unwrap_or_else(GatesModel::default_model);

    // Load score model
    let score_model = lexicon_scaffold::scoring::load_score_model(layout)?;

    // Run gates
    let gate_results = run_all_gates(&gates_model.gates, &layout.root, &[])?;

    // Compute score if model exists
    let score_report = if let Some(ref model) = score_model {
        let dim_results = map_gate_results_to_dimensions(model, &gate_results);
        compute_score(model, &dim_results).ok()
    } else {
        None
    };

    // Write audit record
    let gates_passed = gate_results.iter().all(|r| {
        r.outcome == GateOutcome::Pass || r.outcome == GateOutcome::Skip
    });
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
    })
}

/// Map gate results to scoring dimension results.
fn map_gate_results_to_dimensions(
    model: &ScoreModel,
    gate_results: &[lexicon_gates::result::GateResult],
) -> Vec<DimensionResult> {
    model
        .dimensions
        .iter()
        .map(|dim| {
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
