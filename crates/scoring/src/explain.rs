use crate::engine::{ScoreReport, Verdict};

/// Generate a human-readable explanation of a score report.
pub fn explain_score(report: &ScoreReport) -> String {
    let mut lines = Vec::new();

    let verdict_str = match report.verdict {
        Verdict::Pass => "PASS",
        Verdict::Warn => "WARN",
        Verdict::Fail => "FAIL",
    };

    lines.push(format!(
        "Score: {:.1}% — {}",
        report.total_score * 100.0,
        verdict_str
    ));
    lines.push(String::new());

    lines.push("Dimensions:".to_string());
    for dim in &report.dimensions {
        let status = if dim.passed { "ok" } else { "FAIL" };
        lines.push(format!(
            "  [{status}] {}: {:.1}% — {}",
            dim.dimension_id,
            dim.value * 100.0,
            dim.explanation
        ));
    }

    if !report.all_required_passed {
        lines.push(String::new());
        lines.push("Required gates failed — overall verdict is FAIL regardless of score.".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::DimensionResult;

    #[test]
    fn test_explain_pass() {
        let report = ScoreReport {
            total_score: 0.95,
            all_required_passed: true,
            verdict: Verdict::Pass,
            dimensions: vec![DimensionResult {
                dimension_id: "correctness".to_string(),
                value: 1.0,
                passed: true,
                explanation: "all tests passed".to_string(),
            }],
        };
        let explanation = explain_score(&report);
        assert!(explanation.contains("95.0%"));
        assert!(explanation.contains("PASS"));
        assert!(explanation.contains("correctness"));
    }

    #[test]
    fn test_explain_fail() {
        let report = ScoreReport {
            total_score: 0.3,
            all_required_passed: false,
            verdict: Verdict::Fail,
            dimensions: vec![DimensionResult {
                dimension_id: "correctness".to_string(),
                value: 0.0,
                passed: false,
                explanation: "tests failed".to_string(),
            }],
        };
        let explanation = explain_score(&report);
        assert!(explanation.contains("FAIL"));
        assert!(explanation.contains("Required gates failed"));
    }
}
