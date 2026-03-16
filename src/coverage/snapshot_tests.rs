use crate::coverage::matcher::{ClauseCoverage, ClauseType, ContractCoverage};
use crate::coverage::report::{build_report, format_report};

fn make_clause(
    id: &str,
    clause_type: ClauseType,
    tags: Vec<String>,
    covered: bool,
) -> ClauseCoverage {
    ClauseCoverage {
        clause_id: id.to_string(),
        clause_type,
        description: format!("Description for {id}"),
        expected_tags: tags,
        matched_tests: Vec::new(),
        is_covered: covered,
    }
}

#[test]
fn snapshot_coverage_report() {
    let cov1 = ContractCoverage {
        contract_id: "kv-store".to_string(),
        clauses: vec![
            make_clause(
                "req-001",
                ClauseType::RequiredSemantic,
                vec!["conformance".to_string()],
                true,
            ),
            make_clause(
                "req-002",
                ClauseType::RequiredSemantic,
                vec!["advanced".to_string()],
                false,
            ),
            make_clause(
                "forbid-001",
                ClauseType::ForbiddenSemantic,
                vec!["safety".to_string()],
                true,
            ),
            make_clause(
                "inv-001",
                ClauseType::Invariant,
                Vec::new(),
                false,
            ),
        ],
        covered_count: 2,
        total_count: 3,
        coverage_pct: 66.7,
    };

    let cov2 = ContractCoverage {
        contract_id: "auth-service".to_string(),
        clauses: vec![
            make_clause(
                "req-001",
                ClauseType::RequiredSemantic,
                vec!["auth-basic".to_string()],
                false,
            ),
        ],
        covered_count: 0,
        total_count: 1,
        coverage_pct: 0.0,
    };

    let report = build_report(vec![cov1, cov2]);
    let text = format_report(&report);

    insta::assert_snapshot!(text);
}
