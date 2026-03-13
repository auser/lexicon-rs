use lexicon_core::contract::{contract_list, contract_new_noninteractive};
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

    // 2. Create a contract.
    let contract = contract_new_noninteractive(
        &layout,
        "kv-store".to_string(),
        "KV Store Contract".to_string(),
        "key-value operations".to_string(),
        vec![
            "get after set returns the stored value".to_string(),
            "delete removes the key".to_string(),
        ],
        vec!["get on missing key returns None".to_string()],
        vec!["must not panic on missing key".to_string()],
    )
    .expect("contract_new_noninteractive failed");

    assert_eq!(contract.id, "kv-store");
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

    // 8. contract_list should return ["kv-store"].
    let ids = contract_list(&layout).expect("contract_list failed");
    assert_eq!(ids, vec!["kv-store"]);
}

#[test]
fn test_verify_without_scoring_model() {
    let (_dir, layout) = setup_repo();

    // Create a contract so the repo has something to work with.
    contract_new_noninteractive(
        &layout,
        "basic".to_string(),
        "Basic Contract".to_string(),
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
