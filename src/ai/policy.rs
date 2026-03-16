use crate::spec::manifest::PolicyConfig;

/// Check if AI is allowed to edit a given file path.
pub fn ai_may_edit(policy: &PolicyConfig, path: &str) -> EditPermission {
    for pattern in &policy.ai_protected {
        if glob_match(pattern, path) {
            return EditPermission::Protected;
        }
    }

    for pattern in &policy.ai_requires_review {
        if glob_match(pattern, path) {
            return EditPermission::RequiresReview;
        }
    }

    for pattern in &policy.ai_may_edit {
        if glob_match(pattern, path) {
            return EditPermission::Allowed;
        }
    }

    // Default: require review for unlisted files
    EditPermission::RequiresReview
}

/// Permission level for AI editing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditPermission {
    /// AI may freely edit this file.
    Allowed,
    /// AI changes require manual review.
    RequiresReview,
    /// AI must not edit this file.
    Protected,
}

/// Simple glob matching (supports `*` and `**`).
fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern.contains("**") {
        // `**` matches any number of path segments (including zero)
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');
            let path_matches_prefix = prefix.is_empty() || path.starts_with(prefix);
            if !path_matches_prefix {
                return false;
            }
            // For the suffix part, handle wildcards like `*.rs`
            if suffix.contains('*') {
                let suffix_parts: Vec<&str> = suffix.split('*').collect();
                if suffix_parts.len() == 2 {
                    return path.ends_with(suffix_parts[1]);
                }
            }
            let path_matches_suffix = suffix.is_empty() || path.ends_with(suffix);
            return path_matches_suffix;
        }
    }

    if pattern.contains('*') {
        // Simple wildcard: `*.rs` matches `foo.rs`
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return path.starts_with(parts[0]) && path.ends_with(parts[1]);
        }
    }

    pattern == path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy_permissions() {
        let policy = PolicyConfig::default();

        assert_eq!(ai_may_edit(&policy, "src/lib.rs"), EditPermission::Allowed);
        assert_eq!(
            ai_may_edit(&policy, "tests/conformance/kv.rs"),
            EditPermission::Allowed
        );
        assert_eq!(
            ai_may_edit(&policy, "specs/contracts/kv.toml"),
            EditPermission::RequiresReview
        );
        assert_eq!(
            ai_may_edit(&policy, "CLAUDE.md"),
            EditPermission::RequiresReview
        );
        assert_eq!(
            ai_may_edit(&policy, ".lexicon/manifest.toml"),
            EditPermission::Protected
        );
        assert_eq!(
            ai_may_edit(&policy, "specs/gates.toml"),
            EditPermission::Protected
        );
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("src/**/*.rs", "src/lib.rs"));
        assert!(glob_match("src/**/*.rs", "src/deep/nested/file.rs"));
        assert!(!glob_match("src/**/*.rs", "tests/test.rs"));
        assert!(glob_match("*.toml", "Cargo.toml"));
        assert!(glob_match("CLAUDE.md", "CLAUDE.md"));
        assert!(!glob_match("CLAUDE.md", "README.md"));
    }
}
