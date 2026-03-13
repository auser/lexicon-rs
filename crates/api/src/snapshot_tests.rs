use crate::diff::diff_snapshots;
use crate::report::format_diff_report;
use crate::schema::{ApiItem, ApiItemKind, ApiSnapshot, Visibility};

fn make_item(kind: ApiItemKind, name: &str, sig: &str, vis: Visibility) -> ApiItem {
    ApiItem {
        kind,
        name: name.into(),
        module_path: vec![],
        signature: sig.into(),
        visibility: vis,
        trait_associations: vec![],
        stability: None,
        doc_summary: None,
        span_file: Some("src/lib.rs".into()),
        span_line: Some(1),
    }
}

fn make_snapshot(items: Vec<ApiItem>) -> ApiSnapshot {
    ApiSnapshot {
        crate_name: "test-crate".into(),
        version: Some("0.1.0".into()),
        items,
        extracted_at: "2026-01-01T00:00:00Z".into(),
    }
}

#[test]
fn snapshot_api_diff_report() {
    // Baseline: struct Config, fn process, trait Handler
    let baseline = make_snapshot(vec![
        make_item(
            ApiItemKind::Struct,
            "Config",
            "struct Config",
            Visibility::Public,
        ),
        make_item(
            ApiItemKind::Function,
            "process",
            "fn process(input: &str) -> bool",
            Visibility::Public,
        ),
        make_item(
            ApiItemKind::Trait,
            "Handler",
            "trait Handler",
            Visibility::Public,
        ),
    ]);

    // Current: struct Config (unchanged), fn process (signature changed), trait Handler removed,
    //          fn validate added
    let current = make_snapshot(vec![
        make_item(
            ApiItemKind::Struct,
            "Config",
            "struct Config",
            Visibility::Public,
        ),
        make_item(
            ApiItemKind::Function,
            "process",
            "fn process(input: &str, strict: bool) -> bool",
            Visibility::Public,
        ),
        make_item(
            ApiItemKind::Function,
            "validate",
            "fn validate(data: &[u8]) -> bool",
            Visibility::Public,
        ),
    ]);

    let diff = diff_snapshots(&baseline, &current);
    let report = format_diff_report(&diff);

    insta::assert_snapshot!(report);
}

#[test]
fn snapshot_api_snapshot_json() {
    let snapshot = make_snapshot(vec![
        make_item(
            ApiItemKind::Struct,
            "Config",
            "struct Config",
            Visibility::Public,
        ),
        make_item(
            ApiItemKind::Function,
            "process",
            "fn process(input: &str) -> bool",
            Visibility::Public,
        ),
    ]);

    let json = serde_json::to_string_pretty(&snapshot).unwrap();
    insta::assert_snapshot!(json);
}
