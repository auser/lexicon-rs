//! Report generation for coverage analysis.

use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::coverage::error::CoverageError;
use crate::coverage::matcher::{ClauseType, ContractCoverage};

/// An uncovered clause for inclusion in the report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredClause {
    pub contract_id: String,
    pub clause_id: String,
    pub clause_type: ClauseType,
    pub description: String,
}

/// Aggregated coverage report across all contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Per-contract coverage details.
    pub contracts: Vec<ContractCoverage>,
    /// Overall coverage percentage across all contracts.
    pub overall_coverage_pct: f64,
    /// Total number of covered clauses (with tags) across all contracts.
    pub total_covered: usize,
    /// Total number of clauses with tags across all contracts.
    pub total_clauses: usize,
    /// All uncovered clauses that have tags.
    pub uncovered_clauses: Vec<UncoveredClause>,
}

/// Build an aggregated coverage report from individual contract coverages.
pub fn build_report(coverages: Vec<ContractCoverage>) -> CoverageReport {
    let total_covered: usize = coverages.iter().map(|c| c.covered_count).sum();
    let total_clauses: usize = coverages.iter().map(|c| c.total_count).sum();

    let overall_coverage_pct = if total_clauses > 0 {
        (total_covered as f64 / total_clauses as f64) * 100.0
    } else {
        0.0
    };

    let mut uncovered_clauses = Vec::new();
    for cov in &coverages {
        for clause in &cov.clauses {
            if !clause.expected_tags.is_empty() && !clause.is_covered {
                uncovered_clauses.push(UncoveredClause {
                    contract_id: cov.contract_id.clone(),
                    clause_id: clause.clause_id.clone(),
                    clause_type: clause.clause_type.clone(),
                    description: clause.description.clone(),
                });
            }
        }
    }

    CoverageReport {
        contracts: coverages,
        overall_coverage_pct,
        total_covered,
        total_clauses,
        uncovered_clauses,
    }
}

/// Format a coverage report as a human-readable string.
pub fn format_report(report: &CoverageReport) -> String {
    let mut out = String::new();

    writeln!(out, "=== Lexicon Coverage Report ===").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "Overall coverage: {:.1}% ({}/{})",
        report.overall_coverage_pct, report.total_covered, report.total_clauses
    )
    .unwrap();
    writeln!(out).unwrap();

    for contract in &report.contracts {
        writeln!(
            out,
            "Contract: {} - {:.1}% ({}/{})",
            contract.contract_id,
            contract.coverage_pct,
            contract.covered_count,
            contract.total_count
        )
        .unwrap();

        for clause in &contract.clauses {
            let status = if clause.expected_tags.is_empty() {
                "no tags"
            } else if clause.is_covered {
                "covered"
            } else {
                "UNCOVERED"
            };
            writeln!(
                out,
                "  [{status}] {clause_type} {id}: {desc}",
                clause_type = clause.clause_type,
                id = clause.clause_id,
                desc = clause.description,
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    if !report.uncovered_clauses.is_empty() {
        writeln!(out, "Uncovered clauses:").unwrap();
        for uc in &report.uncovered_clauses {
            writeln!(
                out,
                "  - {}/{} ({}): {}",
                uc.contract_id, uc.clause_id, uc.clause_type, uc.description
            )
            .unwrap();
        }
    }

    out
}

/// Format a coverage report as JSON.
pub fn format_json_report(report: &CoverageReport) -> Result<String, CoverageError> {
    serde_json::to_string_pretty(report).map_err(|e| CoverageError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::coverage::matcher::{ClauseCoverage, ContractCoverage};

    use super::*;

    fn make_clause(id: &str, clause_type: ClauseType, tags: Vec<String>, covered: bool) -> ClauseCoverage {
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
    fn test_build_report_empty() {
        let report = build_report(Vec::new());
        assert_eq!(report.total_covered, 0);
        assert_eq!(report.total_clauses, 0);
        assert_eq!(report.overall_coverage_pct, 0.0);
        assert!(report.uncovered_clauses.is_empty());
    }

    #[test]
    fn test_build_report_with_coverage() {
        let coverage = ContractCoverage {
            contract_id: "test-contract".to_string(),
            clauses: vec![
                make_clause("req-001", ClauseType::RequiredSemantic, vec!["basic".to_string()], true),
                make_clause("req-002", ClauseType::RequiredSemantic, vec!["advanced".to_string()], false),
                make_clause("inv-001", ClauseType::Invariant, Vec::new(), false),
            ],
            covered_count: 1,
            total_count: 2,
            coverage_pct: 50.0,
        };

        let report = build_report(vec![coverage]);
        assert_eq!(report.total_covered, 1);
        assert_eq!(report.total_clauses, 2);
        assert_eq!(report.overall_coverage_pct, 50.0);
        assert_eq!(report.uncovered_clauses.len(), 1);
        assert_eq!(report.uncovered_clauses[0].clause_id, "req-002");
    }

    #[test]
    fn test_build_report_multiple_contracts() {
        let cov1 = ContractCoverage {
            contract_id: "contract-a".to_string(),
            clauses: vec![
                make_clause("req-001", ClauseType::RequiredSemantic, vec!["tag".to_string()], true),
            ],
            covered_count: 1,
            total_count: 1,
            coverage_pct: 100.0,
        };
        let cov2 = ContractCoverage {
            contract_id: "contract-b".to_string(),
            clauses: vec![
                make_clause("req-001", ClauseType::RequiredSemantic, vec!["tag".to_string()], false),
            ],
            covered_count: 0,
            total_count: 1,
            coverage_pct: 0.0,
        };

        let report = build_report(vec![cov1, cov2]);
        assert_eq!(report.total_covered, 1);
        assert_eq!(report.total_clauses, 2);
        assert_eq!(report.overall_coverage_pct, 50.0);
    }

    #[test]
    fn test_format_report_contains_key_info() {
        let coverage = ContractCoverage {
            contract_id: "my-contract".to_string(),
            clauses: vec![
                make_clause("req-001", ClauseType::RequiredSemantic, vec!["basic".to_string()], true),
                make_clause("forbid-001", ClauseType::ForbiddenSemantic, vec!["safety".to_string()], false),
            ],
            covered_count: 1,
            total_count: 2,
            coverage_pct: 50.0,
        };

        let report = build_report(vec![coverage]);
        let text = format_report(&report);

        assert!(text.contains("Overall coverage: 50.0%"));
        assert!(text.contains("my-contract"));
        assert!(text.contains("[covered]"));
        assert!(text.contains("[UNCOVERED]"));
        assert!(text.contains("forbid-001"));
    }

    #[test]
    fn test_format_json_report() {
        let report = build_report(Vec::new());
        let json = format_json_report(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["overall_coverage_pct"], 0.0);
        assert_eq!(parsed["total_covered"], 0);
    }
}
