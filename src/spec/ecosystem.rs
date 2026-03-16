//! Ecosystem manifest types for multi-repo governance.

use serde::{Deserialize, Serialize};

/// Manifest describing a multi-repo ecosystem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcosystemManifest {
    /// Human-readable name for this ecosystem.
    pub name: String,

    /// Repositories that belong to this ecosystem.
    #[serde(default)]
    pub repos: Vec<RepoEntry>,

    /// Contracts shared across all repos in the ecosystem.
    #[serde(default)]
    pub shared_contracts: Vec<String>,

    /// Governance rules that apply ecosystem-wide.
    #[serde(default)]
    pub governance_rules: Vec<String>,
}

/// A repository entry within the ecosystem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoEntry {
    /// Repository name.
    pub name: String,

    /// Path (relative or absolute) to the repository root.
    pub path: String,

    /// Role of this repository in the ecosystem.
    pub role: RepoRole,

    /// Human-readable description.
    #[serde(default)]
    pub description: String,
}

/// The role a repository plays in the ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoRole {
    /// Core platform repository.
    Platform,
    /// Microservice or standalone service.
    Service,
    /// Shared library consumed by other repos.
    Library,
    /// Developer tooling, CLI, or build tools.
    Tool,
    /// Infrastructure-as-code, deployment, CI/CD.
    Infrastructure,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_manifest_serde_roundtrip() {
        let manifest = EcosystemManifest {
            name: "acme-platform".into(),
            repos: vec![
                RepoEntry {
                    name: "acme-core".into(),
                    path: "../acme-core".into(),
                    role: RepoRole::Platform,
                    description: "Core platform services".into(),
                },
                RepoEntry {
                    name: "acme-auth".into(),
                    path: "../acme-auth".into(),
                    role: RepoRole::Service,
                    description: "Authentication service".into(),
                },
            ],
            shared_contracts: vec!["api-versioning".into()],
            governance_rules: vec!["all repos must pass conformance checks".into()],
        };

        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        let back: EcosystemManifest = toml::from_str(&toml_str).unwrap();
        assert_eq!(manifest, back);
    }

    #[test]
    fn test_ecosystem_manifest_defaults() {
        let toml_str = r#"
            name = "my-ecosystem"
        "#;
        let manifest: EcosystemManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.name, "my-ecosystem");
        assert!(manifest.repos.is_empty());
        assert!(manifest.shared_contracts.is_empty());
        assert!(manifest.governance_rules.is_empty());
    }

    #[test]
    fn test_repo_role_serde() {
        for role in [
            RepoRole::Platform,
            RepoRole::Service,
            RepoRole::Library,
            RepoRole::Tool,
            RepoRole::Infrastructure,
        ] {
            let json = serde_json::to_string(&role).unwrap();
            let back: RepoRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, back);
        }
    }

    #[test]
    fn test_repo_entry_toml_roundtrip() {
        let entry = RepoEntry {
            name: "my-tool".into(),
            path: "/opt/tools/my-tool".into(),
            role: RepoRole::Tool,
            description: "Internal developer tool".into(),
        };

        let toml_str = toml::to_string_pretty(&entry).unwrap();
        let back: RepoEntry = toml::from_str(&toml_str).unwrap();
        assert_eq!(entry, back);
    }
}
