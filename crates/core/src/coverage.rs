use lexicon_audit::writer::write_audit_record;
use lexicon_coverage::analyzer::scan_directory;
use lexicon_coverage::matcher::compute_coverage;
use lexicon_coverage::report::{build_report, format_report, format_json_report, CoverageReport};
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::common::{Actor, AuditAction};
use lexicon_spec::contract::Contract;

use crate::error::CoreResult;

/// Run coverage analysis for all contracts.
pub fn coverage_report(layout: &RepoLayout, contracts: &[Contract]) -> CoreResult<CoverageReport> {
    // Scan test directories for tags
    let mut all_tags = Vec::new();

    let test_dirs = [
        layout.conformance_tests_dir(),
        layout.behavior_tests_dir(),
        layout.tests_dir(),
    ];

    for dir in &test_dirs {
        if dir.is_dir() {
            let tags = scan_directory(dir)?;
            all_tags.extend(tags);
        }
    }

    // Compute coverage for each contract
    let coverages: Vec<_> = contracts
        .iter()
        .map(|c| compute_coverage(c, &all_tags))
        .collect();

    let report = build_report(coverages);

    // Audit
    let record = AuditRecord::new(
        AuditAction::CoverageReport,
        Actor::System,
        format!(
            "Coverage report: {:.1}% overall ({}/{} clauses covered)",
            report.overall_coverage_pct, report.total_covered, report.total_clauses
        ),
    );
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(report)
}

/// Format a coverage report as human-readable text.
pub fn coverage_report_text(report: &CoverageReport) -> String {
    format_report(report)
}

/// Format a coverage report as JSON.
pub fn coverage_report_json(report: &CoverageReport) -> CoreResult<String> {
    Ok(format_json_report(report)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::contract::Semantic;
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, RepoLayout) {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        crate::init::init_repo_noninteractive(
            &layout,
            "test".to_string(),
            "test".to_string(),
            lexicon_spec::common::RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();
        (dir, layout)
    }

    #[test]
    fn test_coverage_report_no_tests() {
        let (_dir, layout) = setup_repo();
        let mut contract = Contract::new_draft(
            "test-contract".to_string(),
            "Test".to_string(),
            "scope".to_string(),
        );
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "test semantic".to_string(),
            test_tags: vec!["conformance".to_string()],
        });

        let report = coverage_report(&layout, &[contract]).unwrap();
        assert_eq!(report.total_covered, 0);
        assert_eq!(report.total_clauses, 1);
    }

    #[test]
    fn test_coverage_report_with_tags() {
        let (_dir, layout) = setup_repo();
        let mut contract = Contract::new_draft(
            "test-contract".to_string(),
            "Test".to_string(),
            "scope".to_string(),
        );
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "test semantic".to_string(),
            test_tags: vec!["conformance".to_string()],
        });

        // Create a test file with a matching tag
        let test_dir = layout.conformance_tests_dir();
        std::fs::create_dir_all(&test_dir).unwrap();
        std::fs::write(
            test_dir.join("test_kv.rs"),
            r#"
// lexicon-tag: conformance
#[test]
fn test_basic() {}
"#,
        )
        .unwrap();

        let report = coverage_report(&layout, &[contract]).unwrap();
        assert_eq!(report.total_covered, 1);
        assert_eq!(report.total_clauses, 1);
        assert_eq!(report.overall_coverage_pct, 100.0);
    }
}
