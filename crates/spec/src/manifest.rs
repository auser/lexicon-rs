use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::{ConformanceStyle, NamingConvention, RepoType};
use crate::version::SchemaVersion;

/// Root manifest for a lexicon-managed repository.
///
/// Stored at `.lexicon/manifest.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: SchemaVersion,
    pub project: ProjectMeta,
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub policy: PolicyConfig,
}

/// Metadata about the managed project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub description: String,
    pub repo_type: RepoType,
    /// Domain of the project, e.g. "key-value store", "parser", "web framework".
    pub domain: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User preferences that inform artifact generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub naming_convention: NamingConvention,
    pub conformance_style: ConformanceStyle,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            naming_convention: NamingConvention::KebabCase,
            conformance_style: ConformanceStyle::TraitBased,
        }
    }
}

/// Policy configuration for AI safety and change control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Glob patterns for files AI may edit freely.
    #[serde(default)]
    pub ai_may_edit: Vec<String>,
    /// Glob patterns for files AI changes require manual review.
    #[serde(default)]
    pub ai_requires_review: Vec<String>,
    /// Glob patterns for files AI must never edit.
    #[serde(default)]
    pub ai_protected: Vec<String>,
    /// Whether weakening a gate definition requires explicit approval.
    #[serde(default = "default_true")]
    pub gate_weakening_requires_approval: bool,
    /// Whether deleting tests requires explicit approval.
    #[serde(default = "default_true")]
    pub test_deletion_requires_approval: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            ai_may_edit: vec!["src/**/*.rs".to_string(), "tests/**/*.rs".to_string()],
            ai_requires_review: vec![
                "specs/**/*.toml".to_string(),
                "CLAUDE.md".to_string(),
            ],
            ai_protected: vec![
                ".lexicon/manifest.toml".to_string(),
                "specs/gates.toml".to_string(),
            ],
            gate_weakening_requires_approval: true,
            test_deletion_requires_approval: true,
        }
    }
}

impl Manifest {
    /// Create a new manifest with sensible defaults.
    pub fn new(name: String, description: String, repo_type: RepoType, domain: String) -> Self {
        let now = Utc::now();
        Self {
            schema_version: SchemaVersion::CURRENT,
            project: ProjectMeta {
                name,
                description,
                repo_type,
                domain,
                created_at: now,
                updated_at: now,
            },
            preferences: Preferences::default(),
            policy: PolicyConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_toml_roundtrip() {
        let manifest = Manifest::new(
            "my-lib".to_string(),
            "A test library".to_string(),
            RepoType::Library,
            "key-value store".to_string(),
        );
        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        let parsed: Manifest = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.project.name, "my-lib");
        assert_eq!(parsed.project.domain, "key-value store");
        assert!(parsed.policy.gate_weakening_requires_approval);
    }

    #[test]
    fn test_default_policy() {
        let policy = PolicyConfig::default();
        assert!(policy.gate_weakening_requires_approval);
        assert!(policy.test_deletion_requires_approval);
        assert!(!policy.ai_may_edit.is_empty());
        assert!(!policy.ai_protected.is_empty());
    }
}
