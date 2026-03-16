#[cfg(test)]
mod tests {
    use crate::scoring::engine::{DimensionResult, ScoreReport, Verdict};
    use crate::scoring::explain::explain_score;

    fn make_result(id: &str, value: f64, passed: bool, explanation: &str) -> DimensionResult {
        DimensionResult {
            dimension_id: id.to_string(),
            value,
            passed,
            explanation: explanation.to_string(),
        }
    }

    #[test]
    fn snapshot_explain_passing_score() {
        let report = ScoreReport {
            total_score: 0.92,
            all_required_passed: true,
            verdict: Verdict::Pass,
            dimensions: vec![
                make_result("correctness", 1.0, true, "all tests passed"),
                make_result("conformance-coverage", 0.85, true, "17/20 semantics covered"),
                make_result("behavior-pass-rate", 0.9, true, "18/20 behaviors pass"),
                make_result("lint-quality", 1.0, true, "no warnings"),
                make_result("doc-completeness", 0.7, true, "advisory only"),
                make_result("panic-safety", 0.8, true, "no panics detected"),
            ],
        };
        let explanation = explain_score(&report);
        insta::assert_snapshot!("explain_passing_score", explanation);
    }

    #[test]
    fn snapshot_explain_failing_score() {
        let report = ScoreReport {
            total_score: 0.35,
            all_required_passed: false,
            verdict: Verdict::Fail,
            dimensions: vec![
                make_result("correctness", 0.0, false, "3 tests failed"),
                make_result("conformance-coverage", 0.5, true, "10/20 semantics covered"),
                make_result("behavior-pass-rate", 0.4, false, "8/20 behaviors pass"),
                make_result("lint-quality", 0.6, true, "12 warnings"),
                make_result("doc-completeness", 0.2, false, "advisory only"),
                make_result("panic-safety", 0.0, false, "2 panics detected"),
            ],
        };
        let explanation = explain_score(&report);
        insta::assert_snapshot!("explain_failing_score", explanation);
    }
}
