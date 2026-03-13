use lexicon_api::baseline::{load_baseline, save_baseline, save_current};
use lexicon_api::diff::{diff_snapshots, ApiDiff};
use lexicon_api::extract::extract_from_dir;
use lexicon_api::report::{format_diff_report, format_json_report};
use lexicon_api::schema::ApiSnapshot;
use lexicon_audit::writer::write_audit_record;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::common::{Actor, AuditAction};

use crate::error::CoreResult;

/// Scan the repository source and extract the public API.
pub fn api_scan(layout: &RepoLayout) -> CoreResult<ApiSnapshot> {
    let src_dir = layout.root.join("src");
    let scan_dir = if src_dir.is_dir() {
        src_dir
    } else {
        layout.root.clone()
    };

    let snapshot = extract_from_dir(&scan_dir)?;

    // Save current scan
    std::fs::create_dir_all(layout.api_dir())?;
    let current_path = layout.api_dir().join("current.json");
    save_current(&snapshot, &current_path)?;

    // Audit
    let record = AuditRecord::new(
        AuditAction::ApiScan,
        Actor::System,
        format!("API scan: {} items extracted", snapshot.items.len()),
    );
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(snapshot)
}

/// Diff the current API snapshot against the baseline.
pub fn api_diff(layout: &RepoLayout) -> CoreResult<ApiDiff> {
    let current_path = layout.api_dir().join("current.json");
    let baseline_path = layout.api_dir().join("baseline.json");

    let current = load_baseline(&current_path)?;
    let baseline = load_baseline(&baseline_path)?;

    let diff = diff_snapshots(&baseline, &current);

    // Audit
    let record = AuditRecord::new(
        AuditAction::ApiDiff,
        Actor::System,
        diff.summary(),
    );
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(diff)
}

/// Generate a human-readable API diff report.
pub fn api_report(layout: &RepoLayout) -> CoreResult<String> {
    let diff = api_diff(layout)?;
    Ok(format_diff_report(&diff))
}

/// Generate a JSON API diff report.
pub fn api_report_json(layout: &RepoLayout) -> CoreResult<String> {
    let diff = api_diff(layout)?;
    Ok(format_json_report(&diff)?)
}

/// Save the current API snapshot as the baseline.
pub fn api_baseline(layout: &RepoLayout) -> CoreResult<()> {
    let current_path = layout.api_dir().join("current.json");
    let baseline_path = layout.api_dir().join("baseline.json");

    let current = load_baseline(&current_path)?;
    save_baseline(&current, &baseline_path)?;

    Ok(())
}

/// Check if an API baseline exists.
pub fn has_baseline(layout: &RepoLayout) -> bool {
    layout.api_dir().join("baseline.json").exists()
}

/// Check if a current API scan exists.
pub fn has_current_scan(layout: &RepoLayout) -> bool {
    layout.api_dir().join("current.json").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_api_scan_empty_src() {
        let (dir, layout) = setup_repo();
        // Create a minimal src directory with a Rust file
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn hello() -> &'static str { \"hello\" }\n",
        )
        .unwrap();

        let snapshot = api_scan(&layout).unwrap();
        assert!(!snapshot.items.is_empty());
        assert!(layout.api_dir().join("current.json").exists());
    }

    #[test]
    fn test_api_baseline_roundtrip() {
        let (dir, layout) = setup_repo();
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub struct Foo;\n").unwrap();

        api_scan(&layout).unwrap();
        api_baseline(&layout).unwrap();

        assert!(has_baseline(&layout));
        assert!(has_current_scan(&layout));
    }
}
