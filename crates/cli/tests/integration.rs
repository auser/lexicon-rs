use lexicon_core::api::{api_baseline, api_diff, api_report, api_scan};
use lexicon_core::contract::{contract_list, contract_new_noninteractive};
use lexicon_core::coverage::coverage_report;
use lexicon_core::init::init_repo_noninteractive;
use lexicon_core::score::{gate_init, score_explain, score_init};
use lexicon_core::sync_claude::sync_claude;
use lexicon_core::verify::verify;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::common::RepoType;
use tempfile::TempDir;

/// Helper: initialize a lexicon repo in a tempdir and return the guard + layout.
fn setup_repo() -> (TempDir, RepoLayout) {
    let dir = TempDir::new().expect("failed to create tempdir");
    let layout = RepoLayout::new(dir.path().to_path_buf());
    init_repo_noninteractive(
        &layout,
        "demo-lib".to_string(),
        "A demo library".to_string(),
        RepoType::Library,
        "testing".to_string(),
    )
    .expect("init_repo_noninteractive failed");
    (dir, layout)
}

#[test]
fn test_happy_path_end_to_end() {
    // 1. Create a tempdir and init the repo.
    let (_dir, layout) = setup_repo();

    // 2. Create a contract (ID auto-derived from title).
    let contract = contract_new_noninteractive(
        &layout,
        "KV Store Contract".to_string(),
        String::new(),
        "key-value operations".to_string(),
        vec![
            "get after set returns the stored value".to_string(),
            "delete removes the key".to_string(),
        ],
        vec!["get on missing key returns None".to_string()],
        vec!["must not panic on missing key".to_string()],
    )
    .expect("contract_new_noninteractive failed");

    assert_eq!(contract.id, "kv-store-contract");
    assert_eq!(contract.title, "KV Store Contract");
    assert_eq!(contract.invariants.len(), 2);
    assert_eq!(contract.required_semantics.len(), 1);
    assert_eq!(contract.forbidden_semantics.len(), 1);

    // 3. Initialize scoring model.
    score_init(&layout).expect("score_init failed");
    assert!(layout.scoring_model_path().exists());

    // 4. Initialize gates model.
    gate_init(&layout).expect("gate_init failed");
    assert!(layout.gates_path().exists());

    // 5. score_explain should contain "Scoring Model".
    let explanation = score_explain(&layout).expect("score_explain failed");
    assert!(
        explanation.contains("Scoring Model"),
        "Expected explanation to contain 'Scoring Model', got: {explanation}"
    );

    // 6. verify — gate_results should be non-empty.
    let result = verify(&layout).expect("verify failed");
    assert!(
        !result.gate_results.is_empty(),
        "Expected non-empty gate_results"
    );

    // 7. sync_claude — CLAUDE.md should exist with managed block markers.
    sync_claude(&layout).expect("sync_claude failed");
    let claude_md = std::fs::read_to_string(layout.claude_md_path())
        .expect("failed to read CLAUDE.md");
    assert!(
        claude_md.contains("lexicon:begin:lexicon-context"),
        "CLAUDE.md missing begin marker"
    );
    assert!(
        claude_md.contains("lexicon:end:lexicon-context"),
        "CLAUDE.md missing end marker"
    );

    // 8. contract_list should return the slugified title.
    let ids = contract_list(&layout).expect("contract_list failed");
    assert_eq!(ids, vec!["kv-store-contract"]);
}

#[test]
fn test_verify_without_scoring_model() {
    let (_dir, layout) = setup_repo();

    // Create a contract so the repo has something to work with.
    contract_new_noninteractive(
        &layout,
        "Basic Contract".to_string(),
        String::new(),
        "basic operations".to_string(),
        vec!["must hold".to_string()],
        vec![],
        vec![],
    )
    .expect("contract_new_noninteractive failed");

    // Initialize only gates, not the scoring model.
    gate_init(&layout).expect("gate_init failed");
    assert!(layout.gates_path().exists());
    assert!(!layout.scoring_model_path().exists());

    // verify should still succeed with gates only.
    let result = verify(&layout).expect("verify failed");
    assert!(
        !result.gate_results.is_empty(),
        "Expected non-empty gate_results even without scoring model"
    );
    // score_report should be None since we did not init the scoring model.
    assert!(
        result.score_report.is_none(),
        "Expected no score_report without scoring model"
    );
}

#[test]
fn test_api_scan_diff_report_flow() {
    let (dir, layout) = setup_repo();

    // Create a src/ directory with some public Rust items.
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("failed to create src/");
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
pub struct Config {
    pub name: String,
}

pub fn process(input: &str) -> bool {
    true
}

pub trait Handler {
    fn handle(&self);
}
"#,
    )
    .expect("failed to write lib.rs");

    // 1. Run api_scan — verify items were extracted.
    let snapshot = api_scan(&layout).expect("api_scan failed");
    assert!(
        snapshot.items.len() >= 3,
        "Expected at least 3 API items (struct, fn, trait), got {}",
        snapshot.items.len()
    );

    // 2. Save as baseline.
    api_baseline(&layout).expect("api_baseline failed");
    assert!(
        layout.api_dir().join("baseline.json").exists(),
        "baseline.json should exist after api_baseline"
    );

    // 3. Modify the source: add a new function, remove the trait.
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
pub struct Config {
    pub name: String,
}

pub fn process(input: &str) -> bool {
    true
}

pub fn validate(data: &[u8]) -> bool {
    !data.is_empty()
}
"#,
    )
    .expect("failed to update lib.rs");

    // 4. Scan again.
    let snapshot2 = api_scan(&layout).expect("second api_scan failed");
    // Should have struct + process + validate = 3 items (trait removed).
    assert!(
        snapshot2.items.len() >= 3,
        "Expected at least 3 API items after modification, got {}",
        snapshot2.items.len()
    );

    // 5. Diff — should detect added and removed items.
    let diff = api_diff(&layout).expect("api_diff failed");
    assert!(
        !diff.added.is_empty(),
        "Expected at least one added item (validate)"
    );
    assert!(
        !diff.removed.is_empty(),
        "Expected at least one removed item (Handler trait)"
    );

    // 6. Report — should be non-empty.
    let report = api_report(&layout).expect("api_report failed");
    assert!(
        !report.is_empty(),
        "Expected non-empty API diff report"
    );
    assert!(
        report.contains("BREAKING") || report.contains("ADDITIVE"),
        "Report should contain change classifications"
    );
}

#[test]
fn test_coverage_analysis_flow() {
    let (_dir, layout) = setup_repo();

    // Create a contract with required_semantics that have test_tags.
    let mut contract = contract_new_noninteractive(
        &layout,
        "Cache Contract".to_string(),
        String::new(),
        "caching operations".to_string(),
        vec!["cache eviction respects capacity".to_string()],
        vec![
            "get returns cached value".to_string(),
            "set stores a value".to_string(),
        ],
        vec!["must not lose data silently".to_string()],
    )
    .expect("contract_new_noninteractive failed");

    // The builder sets test_tags to ["conformance"] for required_semantics
    // and ["safety"] for forbidden_semantics. Invariants get no tags.
    // Add a specific tag to one required semantic to test partial coverage.
    contract.required_semantics[0].test_tags = vec!["cache-get".to_string()];
    contract.required_semantics[1].test_tags = vec!["cache-set".to_string()];

    // Create conformance test files with matching tags.
    let test_dir = layout.conformance_tests_dir();
    std::fs::create_dir_all(&test_dir).expect("failed to create conformance tests dir");
    std::fs::write(
        test_dir.join("test_cache.rs"),
        r#"
// lexicon-tag: cache-get
#[test]
fn test_cache_get() {
    // verifies get returns cached value
}
"#,
    )
    .expect("failed to write test file");

    // Run coverage — should detect partial coverage.
    let report = coverage_report(&layout, &[contract]).expect("coverage_report failed");

    // One of two required semantics is covered (cache-get), the other (cache-set) is not.
    // Forbidden semantic ("safety" tag) is also uncovered. Invariants have no tags.
    assert!(
        report.total_clauses > 0,
        "Expected some clauses with tags"
    );
    assert!(
        report.total_covered >= 1,
        "Expected at least one covered clause"
    );
    assert!(
        report.total_covered < report.total_clauses,
        "Expected partial coverage, not full"
    );
    assert!(
        report.overall_coverage_pct > 0.0 && report.overall_coverage_pct < 100.0,
        "Expected partial coverage percentage, got {:.1}%",
        report.overall_coverage_pct
    );
}

#[test]
fn test_verify_with_coverage_and_api() {
    let (dir, layout) = setup_repo();

    // Create a contract with tagged semantics.
    let _contract = contract_new_noninteractive(
        &layout,
        "Auth Contract".to_string(),
        String::new(),
        "authentication operations".to_string(),
        vec![],
        vec!["login returns token".to_string()],
        vec![],
    )
    .expect("contract_new_noninteractive failed");

    // Create test files with matching conformance tags.
    let test_dir = layout.conformance_tests_dir();
    std::fs::create_dir_all(&test_dir).expect("failed to create conformance tests dir");
    std::fs::write(
        test_dir.join("test_auth.rs"),
        r#"
// lexicon-tag: conformance
#[test]
fn test_login_token() {}
"#,
    )
    .expect("failed to write test file");

    // Create a src/ with API items and set up a baseline so verify detects API drift.
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("failed to create src/");
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn login(user: &str) -> String { user.to_string() }\n",
    )
    .expect("failed to write lib.rs");

    // Scan and save baseline.
    api_scan(&layout).expect("api_scan failed");
    api_baseline(&layout).expect("api_baseline failed");

    // Initialize gates so verify has something to run.
    gate_init(&layout).expect("gate_init failed");

    // Run verify — should populate coverage_report and api_diff.
    let result = verify(&layout).expect("verify failed");

    assert!(
        result.coverage_report.is_some(),
        "Expected coverage_report to be populated"
    );
    let cov = result.coverage_report.as_ref().unwrap();
    assert!(
        cov.total_clauses > 0,
        "Expected at least one clause in coverage report"
    );

    assert!(
        result.api_diff.is_some(),
        "Expected api_diff to be populated when baseline exists"
    );
    let diff = result.api_diff.as_ref().unwrap();
    // Since we scanned and baselined the same source, diff should be empty.
    assert!(
        diff.is_empty(),
        "Expected no API drift when source is unchanged"
    );
}
