use serde::{Deserialize, Serialize};

use crate::schema::{ApiItem, ApiItemKind, ApiSnapshot, Visibility};

/// A change detail describing what specifically changed about an API item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeDetail {
    SignatureChanged { old: String, new: String },
    VisibilityChanged { old: Visibility, new: Visibility },
}

/// A changed API item with details about what changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChange {
    pub name: String,
    pub module_path: Vec<String>,
    pub kind: ApiItemKind,
    pub changes: Vec<ChangeDetail>,
}

/// The breaking level of a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingLevel {
    Breaking,
    Dangerous,
    Additive,
    Unchanged,
}

impl std::fmt::Display for BreakingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Breaking => write!(f, "BREAKING"),
            Self::Dangerous => write!(f, "DANGEROUS"),
            Self::Additive => write!(f, "ADDITIVE"),
            Self::Unchanged => write!(f, "UNCHANGED"),
        }
    }
}

/// The diff between two API snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDiff {
    pub added: Vec<ApiItem>,
    pub removed: Vec<ApiItem>,
    pub changed: Vec<ApiChange>,
}

/// Key used to match items across snapshots.
type ItemKey = (ApiItemKind, String, Vec<String>);

fn item_key(item: &ApiItem) -> ItemKey {
    (item.kind.clone(), item.name.clone(), item.module_path.clone())
}

/// Compare two API snapshots and produce a diff.
pub fn diff_snapshots(baseline: &ApiSnapshot, current: &ApiSnapshot) -> ApiDiff {
    let baseline_keys: std::collections::HashMap<ItemKey, &ApiItem> =
        baseline.items.iter().map(|i| (item_key(i), i)).collect();
    let current_keys: std::collections::HashMap<ItemKey, &ApiItem> =
        current.items.iter().map(|i| (item_key(i), i)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    // Items only in current -> added
    for (key, item) in &current_keys {
        if !baseline_keys.contains_key(key) {
            added.push((*item).clone());
        }
    }

    // Items only in baseline -> removed
    for (key, item) in &baseline_keys {
        if !current_keys.contains_key(key) {
            removed.push((*item).clone());
        }
    }

    // Items in both but potentially changed
    for (key, baseline_item) in &baseline_keys {
        if let Some(current_item) = current_keys.get(key) {
            let mut changes = Vec::new();

            if baseline_item.signature != current_item.signature {
                changes.push(ChangeDetail::SignatureChanged {
                    old: baseline_item.signature.clone(),
                    new: current_item.signature.clone(),
                });
            }

            if baseline_item.visibility != current_item.visibility {
                changes.push(ChangeDetail::VisibilityChanged {
                    old: baseline_item.visibility.clone(),
                    new: current_item.visibility.clone(),
                });
            }

            if !changes.is_empty() {
                changed.push(ApiChange {
                    name: baseline_item.name.clone(),
                    module_path: baseline_item.module_path.clone(),
                    kind: baseline_item.kind.clone(),
                    changes,
                });
            }
        }
    }

    ApiDiff { added, removed, changed }
}

/// Classify the breaking level of a change.
pub fn classify_change(change: &ApiChange) -> BreakingLevel {
    let mut level = BreakingLevel::Unchanged;

    for detail in &change.changes {
        let detail_level = match detail {
            ChangeDetail::VisibilityChanged { old, new } => {
                if is_visibility_narrowed(old, new) {
                    BreakingLevel::Breaking
                } else {
                    BreakingLevel::Additive
                }
            }
            ChangeDetail::SignatureChanged { .. } => BreakingLevel::Dangerous,
        };

        level = max_breaking(level, detail_level);
    }

    level
}

fn visibility_rank(v: &Visibility) -> u8 {
    match v {
        Visibility::Public => 3,
        Visibility::Crate => 2,
        Visibility::Restricted => 1,
        Visibility::Private => 0,
    }
}

fn is_visibility_narrowed(old: &Visibility, new: &Visibility) -> bool {
    visibility_rank(new) < visibility_rank(old)
}

fn max_breaking(a: BreakingLevel, b: BreakingLevel) -> BreakingLevel {
    fn rank(l: BreakingLevel) -> u8 {
        match l {
            BreakingLevel::Unchanged => 0,
            BreakingLevel::Additive => 1,
            BreakingLevel::Dangerous => 2,
            BreakingLevel::Breaking => 3,
        }
    }
    if rank(a) >= rank(b) { a } else { b }
}

impl ApiDiff {
    /// Returns true if there are no differences.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Returns true if any change is breaking.
    pub fn has_breaking(&self) -> bool {
        if !self.removed.is_empty() {
            return true;
        }
        self.changed.iter().any(|c| classify_change(c) == BreakingLevel::Breaking)
    }

    /// Count the number of breaking changes.
    pub fn breaking_count(&self) -> usize {
        let removed_count = self.removed.len();
        let changed_breaking = self.changed.iter()
            .filter(|c| classify_change(c) == BreakingLevel::Breaking)
            .count();
        removed_count + changed_breaking
    }

    /// Generate a summary string.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("{} added", self.added.len()));
        }
        if !self.removed.is_empty() {
            parts.push(format!("{} removed", self.removed.len()));
        }
        if !self.changed.is_empty() {
            parts.push(format!("{} changed", self.changed.len()));
        }
        if parts.is_empty() {
            "No API changes".into()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(name: &str, sig: &str, vis: Visibility) -> ApiItem {
        ApiItem {
            kind: ApiItemKind::Function,
            name: name.into(),
            module_path: vec![],
            signature: sig.into(),
            visibility: vis,
            trait_associations: vec![],
            stability: None,
            doc_summary: None,
            span_file: None,
            span_line: None,
        }
    }

    fn make_snapshot(items: Vec<ApiItem>) -> ApiSnapshot {
        ApiSnapshot {
            crate_name: "test".into(),
            version: None,
            items,
            extracted_at: "now".into(),
        }
    }

    #[test]
    fn diff_added_items() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
            make_item("bar", "fn bar()", Visibility::Public),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "bar");
        assert!(diff.removed.is_empty());
        assert!(diff.changed.is_empty());
    }

    #[test]
    fn diff_removed_items() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
            make_item("bar", "fn bar()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "bar");
        assert!(diff.has_breaking());
    }

    #[test]
    fn diff_changed_signature() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("foo", "fn foo(x: i32)", Visibility::Public),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].name, "foo");
        assert_eq!(classify_change(&diff.changed[0]), BreakingLevel::Dangerous);
    }

    #[test]
    fn diff_changed_visibility() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Crate),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(classify_change(&diff.changed[0]), BreakingLevel::Breaking);
        assert!(diff.has_breaking());
    }

    #[test]
    fn diff_no_changes() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        assert!(diff.is_empty());
        assert!(!diff.has_breaking());
        assert_eq!(diff.breaking_count(), 0);
        assert_eq!(diff.summary(), "No API changes");
    }

    #[test]
    fn summary_format() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("bar", "fn bar()", Visibility::Public),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        let summary = diff.summary();
        assert!(summary.contains("added"));
        assert!(summary.contains("removed"));
    }

    #[test]
    fn breaking_count_includes_removed_and_breaking_changes() {
        let baseline = make_snapshot(vec![
            make_item("foo", "fn foo()", Visibility::Public),
            make_item("bar", "fn bar()", Visibility::Public),
        ]);
        let current = make_snapshot(vec![
            make_item("bar", "fn bar()", Visibility::Crate),
        ]);
        let diff = diff_snapshots(&baseline, &current);
        // foo removed (breaking) + bar visibility narrowed (breaking) = 2
        assert_eq!(diff.breaking_count(), 2);
    }
}
