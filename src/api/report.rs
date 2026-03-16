use super::diff::{ApiChange, ApiDiff, BreakingLevel, ChangeDetail, classify_change};
use super::error::ApiError;

/// Format a human-readable diff report.
pub fn format_diff_report(diff: &ApiDiff) -> String {
    if diff.is_empty() {
        return "No API changes detected.".into();
    }

    let mut report = String::new();
    report.push_str(&format!("API Diff Summary: {}\n", diff.summary()));
    report.push_str(&"=".repeat(60));
    report.push('\n');

    // Group changes by breaking level
    if !diff.removed.is_empty() {
        report.push_str("\n[BREAKING] Removed items:\n");
        for item in &diff.removed {
            report.push_str(&format!(
                "  - {} {} ({})\n",
                item.kind, item.name, item.visibility
            ));
            if !item.module_path.is_empty() {
                report.push_str(&format!("    path: {}\n", item.module_path.join("::")));
            }
        }
    }

    // Separate changed items by breaking level
    let breaking_changes: Vec<&ApiChange> = diff.changed.iter()
        .filter(|c| classify_change(c) == BreakingLevel::Breaking)
        .collect();
    let dangerous_changes: Vec<&ApiChange> = diff.changed.iter()
        .filter(|c| classify_change(c) == BreakingLevel::Dangerous)
        .collect();
    let additive_changes: Vec<&ApiChange> = diff.changed.iter()
        .filter(|c| classify_change(c) == BreakingLevel::Additive)
        .collect();

    if !breaking_changes.is_empty() {
        report.push_str("\n[BREAKING] Changed items:\n");
        for change in &breaking_changes {
            format_change(&mut report, change);
        }
    }

    if !dangerous_changes.is_empty() {
        report.push_str("\n[DANGEROUS] Changed items:\n");
        for change in &dangerous_changes {
            format_change(&mut report, change);
        }
    }

    if !additive_changes.is_empty() {
        report.push_str("\n[ADDITIVE] Changed items:\n");
        for change in &additive_changes {
            format_change(&mut report, change);
        }
    }

    if !diff.added.is_empty() {
        report.push_str("\n[ADDITIVE] Added items:\n");
        for item in &diff.added {
            report.push_str(&format!(
                "  + {} {} ({})\n",
                item.kind, item.name, item.visibility
            ));
            if !item.module_path.is_empty() {
                report.push_str(&format!("    path: {}\n", item.module_path.join("::")));
            }
        }
    }

    report
}

fn format_change(report: &mut String, change: &ApiChange) {
    report.push_str(&format!("  ~ {} {}\n", change.kind, change.name));
    if !change.module_path.is_empty() {
        report.push_str(&format!("    path: {}\n", change.module_path.join("::")));
    }
    for detail in &change.changes {
        match detail {
            ChangeDetail::SignatureChanged { old, new } => {
                report.push_str(&format!("    signature: {old} -> {new}\n"));
            }
            ChangeDetail::VisibilityChanged { old, new } => {
                report.push_str(&format!("    visibility: {old} -> {new}\n"));
            }
        }
    }
}

/// Format a JSON diff report.
pub fn format_json_report(diff: &ApiDiff) -> Result<String, ApiError> {
    let json = serde_json::to_string_pretty(diff)?;
    Ok(json)
}
