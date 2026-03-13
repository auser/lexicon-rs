use similar::{ChangeTag, TextDiff};

/// Generate a unified diff between two strings.
pub fn unified_diff(old: &str, new: &str, old_label: &str, new_label: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    diff.unified_diff()
        .header(old_label, new_label)
        .to_string()
}

/// Returns true if two strings have any differences.
pub fn has_changes(old: &str, new: &str) -> bool {
    let diff = TextDiff::from_lines(old, new);
    diff.iter_all_changes()
        .any(|change| change.tag() != ChangeTag::Equal)
}

/// Summary statistics for a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: usize,
}

/// Compute diff statistics between two strings.
pub fn diff_stats(old: &str, new: &str) -> DiffStats {
    let diff = TextDiff::from_lines(old, new);
    let mut stats = DiffStats {
        additions: 0,
        deletions: 0,
        unchanged: 0,
    };
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => stats.additions += 1,
            ChangeTag::Delete => stats.deletions += 1,
            ChangeTag::Equal => stats.unchanged += 1,
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_diff() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";
        let diff = unified_diff(old, new, "old.txt", "new.txt");
        assert!(diff.contains("--- old.txt"));
        assert!(diff.contains("+++ new.txt"));
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
    }

    #[test]
    fn test_has_changes() {
        assert!(!has_changes("same\n", "same\n"));
        assert!(has_changes("old\n", "new\n"));
    }

    #[test]
    fn test_diff_stats() {
        let old = "a\nb\nc\n";
        let new = "a\nB\nc\nd\n";
        let stats = diff_stats(old, new);
        assert_eq!(stats.deletions, 1); // "b" removed
        assert_eq!(stats.additions, 2); // "B" and "d" added
        assert_eq!(stats.unchanged, 2); // "a" and "c" unchanged
    }

    #[test]
    fn test_no_changes() {
        let stats = diff_stats("same\n", "same\n");
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
        assert_eq!(stats.unchanged, 1);
    }
}
