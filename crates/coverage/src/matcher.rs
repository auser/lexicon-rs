//! Match test tags to contract clauses and compute coverage.

use serde::{Deserialize, Serialize};

use crate::analyzer::TestTag;
use lexicon_spec::contract::Contract;

/// The type of a contract clause.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClauseType {
    Invariant,
    RequiredSemantic,
    ForbiddenSemantic,
    EdgeCase,
}

impl std::fmt::Display for ClauseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClauseType::Invariant => write!(f, "Invariant"),
            ClauseType::RequiredSemantic => write!(f, "Required Semantic"),
            ClauseType::ForbiddenSemantic => write!(f, "Forbidden Semantic"),
            ClauseType::EdgeCase => write!(f, "Edge Case"),
        }
    }
}

/// Coverage information for a single contract clause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseCoverage {
    /// The clause identifier within the contract.
    pub clause_id: String,
    /// What kind of clause this is.
    pub clause_type: ClauseType,
    /// Human-readable description of the clause.
    pub description: String,
    /// Tags declared on the clause (from `test_tags`).
    pub expected_tags: Vec<String>,
    /// Tests whose tags match this clause.
    pub matched_tests: Vec<TestTag>,
    /// Whether at least one test covers this clause.
    pub is_covered: bool,
}

/// Coverage summary for an entire contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCoverage {
    /// The contract identifier.
    pub contract_id: String,
    /// Coverage details per clause.
    pub clauses: Vec<ClauseCoverage>,
    /// Number of clauses with tags that have at least one matching test.
    pub covered_count: usize,
    /// Total number of clauses that have tags (denominator for coverage %).
    pub total_count: usize,
    /// Coverage percentage (covered / total * 100). 0.0 if no clauses have tags.
    pub coverage_pct: f64,
}

/// Compute coverage for a contract given a set of discovered test tags.
///
/// A clause is considered covered if at least one `TestTag` matches any of
/// the clause's `expected_tags`. Clauses without any expected tags are
/// excluded from the coverage percentage denominator.
pub fn compute_coverage(contract: &Contract, tags: &[TestTag]) -> ContractCoverage {
    let mut clauses = Vec::new();

    // Invariants: use their test_tags field.
    for inv in &contract.invariants {
        let matched = find_matching_tests(&inv.test_tags, tags);
        let is_covered = !matched.is_empty() && !inv.test_tags.is_empty();
        clauses.push(ClauseCoverage {
            clause_id: inv.id.clone(),
            clause_type: ClauseType::Invariant,
            description: inv.description.clone(),
            expected_tags: inv.test_tags.clone(),
            matched_tests: matched,
            is_covered,
        });
    }

    // Required semantics: use their test_tags field.
    for sem in &contract.required_semantics {
        let matched = find_matching_tests(&sem.test_tags, tags);
        let is_covered = !matched.is_empty() && !sem.test_tags.is_empty();
        clauses.push(ClauseCoverage {
            clause_id: sem.id.clone(),
            clause_type: ClauseType::RequiredSemantic,
            description: sem.description.clone(),
            expected_tags: sem.test_tags.clone(),
            matched_tests: matched,
            is_covered,
        });
    }

    // Forbidden semantics: use their test_tags field.
    for sem in &contract.forbidden_semantics {
        let matched = find_matching_tests(&sem.test_tags, tags);
        let is_covered = !matched.is_empty() && !sem.test_tags.is_empty();
        clauses.push(ClauseCoverage {
            clause_id: sem.id.clone(),
            clause_type: ClauseType::ForbiddenSemantic,
            description: sem.description.clone(),
            expected_tags: sem.test_tags.clone(),
            matched_tests: matched,
            is_covered,
        });
    }

    // Edge cases: currently don't have test_tags.
    for ec in &contract.edge_cases {
        clauses.push(ClauseCoverage {
            clause_id: ec.id.clone(),
            clause_type: ClauseType::EdgeCase,
            description: ec.scenario.clone(),
            expected_tags: Vec::new(),
            matched_tests: Vec::new(),
            is_covered: false,
        });
    }

    // Compute coverage: only count clauses that have at least one expected tag.
    let with_tags: Vec<&ClauseCoverage> = clauses
        .iter()
        .filter(|c| !c.expected_tags.is_empty())
        .collect();
    let total_count = with_tags.len();
    let covered_count = with_tags.iter().filter(|c| c.is_covered).count();
    let coverage_pct = if total_count > 0 {
        (covered_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    ContractCoverage {
        contract_id: contract.id.clone(),
        clauses,
        covered_count,
        total_count,
        coverage_pct,
    }
}

/// Find test tags that match any of the expected tags.
fn find_matching_tests(expected: &[String], tags: &[TestTag]) -> Vec<TestTag> {
    tags.iter()
        .filter(|t| expected.contains(&t.tag))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use lexicon_spec::contract::{EdgeCase, Invariant, Semantic};

    use super::*;

    fn make_contract() -> Contract {
        Contract::new_draft(
            "test-contract".to_string(),
            "Test Contract".to_string(),
            "Test scope".to_string(),
        )
    }

    fn make_tag(tag: &str, test_name: &str) -> TestTag {
        TestTag {
            tag: tag.to_string(),
            test_name: test_name.to_string(),
            file_path: "test.rs".to_string(),
            line: 1,
        }
    }

    #[test]
    fn test_empty_contract() {
        let contract = make_contract();
        let coverage = compute_coverage(&contract, &[]);
        assert_eq!(coverage.covered_count, 0);
        assert_eq!(coverage.total_count, 0);
        assert_eq!(coverage.coverage_pct, 0.0);
        assert!(coverage.clauses.is_empty());
    }

    #[test]
    fn test_invariants_have_no_tags() {
        let mut contract = make_contract();
        contract.invariants.push(Invariant {
            id: "inv-001".to_string(),
            description: "Always holds".to_string(),
            severity: Default::default(),
            test_tags: Vec::new(),
        });

        let tags = vec![make_tag("conformance", "test_one")];
        let coverage = compute_coverage(&contract, &tags);

        assert_eq!(coverage.clauses.len(), 1);
        assert!(!coverage.clauses[0].is_covered);
        // Invariants don't have tags, so they don't affect the denominator.
        assert_eq!(coverage.total_count, 0);
    }

    #[test]
    fn test_required_semantic_covered() {
        let mut contract = make_contract();
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "Returns None for missing keys".to_string(),
            test_tags: vec!["conformance".to_string(), "basic".to_string()],
        });

        let tags = vec![make_tag("conformance", "test_missing_key")];
        let coverage = compute_coverage(&contract, &tags);

        assert_eq!(coverage.covered_count, 1);
        assert_eq!(coverage.total_count, 1);
        assert_eq!(coverage.coverage_pct, 100.0);
        assert!(coverage.clauses[0].is_covered);
        assert_eq!(coverage.clauses[0].matched_tests.len(), 1);
    }

    #[test]
    fn test_forbidden_semantic_uncovered() {
        let mut contract = make_contract();
        contract.forbidden_semantics.push(Semantic {
            id: "forbid-001".to_string(),
            description: "Must not panic".to_string(),
            test_tags: vec!["safety".to_string()],
        });

        let tags = vec![make_tag("conformance", "test_one")];
        let coverage = compute_coverage(&contract, &tags);

        assert_eq!(coverage.covered_count, 0);
        assert_eq!(coverage.total_count, 1);
        assert_eq!(coverage.coverage_pct, 0.0);
        assert!(!coverage.clauses[0].is_covered);
    }

    #[test]
    fn test_mixed_coverage() {
        let mut contract = make_contract();
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "Covered semantic".to_string(),
            test_tags: vec!["basic".to_string()],
        });
        contract.required_semantics.push(Semantic {
            id: "req-002".to_string(),
            description: "Uncovered semantic".to_string(),
            test_tags: vec!["advanced".to_string()],
        });
        contract.edge_cases.push(EdgeCase {
            id: "edge-001".to_string(),
            scenario: "Empty input".to_string(),
            expected_behavior: "Returns error".to_string(),
        });

        let tags = vec![make_tag("basic", "test_basic")];
        let coverage = compute_coverage(&contract, &tags);

        // 2 semantics with tags, 1 edge case without tags.
        assert_eq!(coverage.total_count, 2);
        assert_eq!(coverage.covered_count, 1);
        assert_eq!(coverage.coverage_pct, 50.0);
        assert_eq!(coverage.clauses.len(), 3);
    }

    #[test]
    fn test_multiple_tags_match() {
        let mut contract = make_contract();
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "Multiple tag clause".to_string(),
            test_tags: vec!["tag-a".to_string(), "tag-b".to_string()],
        });

        let tags = vec![
            make_tag("tag-a", "test_a"),
            make_tag("tag-b", "test_b"),
            make_tag("tag-c", "test_c"),
        ];
        let coverage = compute_coverage(&contract, &tags);

        assert!(coverage.clauses[0].is_covered);
        assert_eq!(coverage.clauses[0].matched_tests.len(), 2);
    }
}
