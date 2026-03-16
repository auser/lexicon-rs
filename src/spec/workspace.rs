//! Workspace manifest and dependency rule types for multi-crate workspaces.

use serde::{Deserialize, Serialize};

use super::mode::OperatingMode;

/// Manifest describing a multi-crate workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    /// The operating mode (should be Workspace for this manifest).
    #[serde(default)]
    pub mode: OperatingMode,

    /// Roles assigned to each crate in the workspace.
    #[serde(default)]
    pub crate_roles: Vec<CrateRole>,

    /// Rules governing allowed/forbidden dependencies between crate roles.
    #[serde(default)]
    pub dependency_rules: Vec<DependencyRule>,

    /// Names of contracts shared across the workspace.
    #[serde(default)]
    pub shared_contracts: Vec<String>,
}

/// A crate's role within the workspace architecture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrateRole {
    /// Crate name (e.g. `lexicon-spec`).
    pub name: String,

    /// Architectural role.
    pub role: CrateRoleKind,

    /// Human-readable description of the crate's purpose.
    #[serde(default)]
    pub description: String,
}

/// Architectural role for a crate within a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrateRoleKind {
    /// Core/foundational crate that others depend on.
    Foundation,
    /// Defines interfaces/traits without implementation.
    Interface,
    /// Adapts external systems to internal interfaces.
    Adapter,
    /// Top-level application crate (binary, CLI, server).
    Application,
    /// Shared utilities without domain logic.
    Utility,
    /// Test-only crate (test harness, fixtures, helpers).
    Test,
}

/// A rule governing dependencies between crate roles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyRule {
    /// The role that this rule applies to (the dependent).
    pub from_role: CrateRoleKind,

    /// Roles that `from_role` is allowed to depend on.
    #[serde(default)]
    pub allowed_targets: Vec<CrateRoleKind>,

    /// Roles that `from_role` must never depend on.
    #[serde(default)]
    pub forbidden_targets: Vec<CrateRoleKind>,

    /// Human-readable explanation for this rule.
    #[serde(default)]
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_manifest_serde_roundtrip() {
        let manifest = WorkspaceManifest {
            mode: OperatingMode::Workspace,
            crate_roles: vec![
                CrateRole {
                    name: "lexicon-spec".into(),
                    role: CrateRoleKind::Foundation,
                    description: "Domain types and schemas".into(),
                },
                CrateRole {
                    name: "lexicon-cli".into(),
                    role: CrateRoleKind::Application,
                    description: "CLI entry point".into(),
                },
            ],
            dependency_rules: vec![DependencyRule {
                from_role: CrateRoleKind::Application,
                allowed_targets: vec![
                    CrateRoleKind::Foundation,
                    CrateRoleKind::Interface,
                    CrateRoleKind::Utility,
                ],
                forbidden_targets: vec![CrateRoleKind::Test],
                description: "Applications can depend on anything except test crates".into(),
            }],
            shared_contracts: vec!["api-stability".into(), "error-handling".into()],
        };

        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        let back: WorkspaceManifest = toml::from_str(&toml_str).unwrap();
        assert_eq!(manifest, back);
    }

    #[test]
    fn test_workspace_manifest_defaults() {
        let toml_str = r#"
            mode = "workspace"
        "#;
        let manifest: WorkspaceManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.mode, OperatingMode::Workspace);
        assert!(manifest.crate_roles.is_empty());
        assert!(manifest.dependency_rules.is_empty());
        assert!(manifest.shared_contracts.is_empty());
    }

    #[test]
    fn test_crate_role_kind_serde() {
        for kind in [
            CrateRoleKind::Foundation,
            CrateRoleKind::Interface,
            CrateRoleKind::Adapter,
            CrateRoleKind::Application,
            CrateRoleKind::Utility,
            CrateRoleKind::Test,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            let back: CrateRoleKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn test_dependency_rule_serde_roundtrip() {
        let rule = DependencyRule {
            from_role: CrateRoleKind::Foundation,
            allowed_targets: vec![CrateRoleKind::Utility],
            forbidden_targets: vec![
                CrateRoleKind::Application,
                CrateRoleKind::Adapter,
            ],
            description: "Foundation crates must not depend on application or adapter crates"
                .into(),
        };

        let json = serde_json::to_string_pretty(&rule).unwrap();
        let back: DependencyRule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, back);
    }
}
