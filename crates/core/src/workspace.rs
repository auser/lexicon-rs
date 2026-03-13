//! Workspace governance initialization and health checks.

use std::fs;

use lexicon_repo::detect::{detect_shape, RepoShape};
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::mode::OperatingMode;
use lexicon_spec::workspace::{
    CrateRole, CrateRoleKind, DependencyRule, WorkspaceManifest,
};

use crate::error::{CoreError, CoreResult};

/// Result of workspace verification.
#[derive(Debug)]
pub struct WorkspaceVerifyResult {
    pub issues: Vec<String>,
    pub passed: bool,
}

/// Initialize workspace governance.
///
/// Requires that `lexicon init` has already been run. Detects the Cargo workspace
/// members and auto-assigns crate roles based on naming conventions.
pub fn workspace_init(layout: &RepoLayout) -> CoreResult<()> {
    // Verify .lexicon/ exists
    if !layout.lexicon_dir().is_dir() {
        return Err(CoreError::Other(
            "No .lexicon/ directory found. Run `lexicon init` first.".into(),
        ));
    }

    // Detect repo shape — must be a workspace
    let shape = detect_shape(&layout.root);
    let member_count = match shape {
        RepoShape::Workspace { member_count } => member_count,
        _ => {
            return Err(CoreError::Other(
                "This repository is not a Cargo workspace. \
                 Workspace governance requires a [workspace] section in Cargo.toml."
                    .into(),
            ));
        }
    };

    // Parse workspace members from Cargo.toml
    let members = parse_workspace_members(&layout.root)?;

    // Auto-assign crate roles based on naming conventions
    let crate_roles: Vec<CrateRole> = members
        .iter()
        .map(|name| CrateRole {
            name: name.clone(),
            role: infer_role_from_name(name),
            description: String::new(),
        })
        .collect();

    // Build the workspace manifest
    let manifest = WorkspaceManifest {
        mode: OperatingMode::Workspace,
        crate_roles,
        dependency_rules: default_dependency_rules(),
        shared_contracts: Vec::new(),
    };

    // Write workspace.toml
    let toml_str = toml::to_string_pretty(&manifest)
        .map_err(|e| CoreError::Other(format!("Failed to serialize workspace manifest: {e}")))?;
    fs::write(layout.workspace_manifest_path(), toml_str)?;

    // Create architecture directory and default rules
    fs::create_dir_all(layout.architecture_dir())?;

    // Wrap rules in a table for valid TOML serialization
    let rules_wrapper = std::collections::BTreeMap::from([
        ("rules".to_string(), &manifest.dependency_rules),
    ]);
    let rules_toml = toml::to_string_pretty(&rules_wrapper)
        .map_err(|e| CoreError::Other(format!("Failed to serialize dependency rules: {e}")))?;
    fs::write(layout.architecture_rules_path(), rules_toml)?;

    eprintln!(
        "Workspace governance initialized with {} crate(s).",
        member_count
    );

    Ok(())
}

/// Verify workspace architecture.
pub fn workspace_verify(layout: &RepoLayout) -> CoreResult<WorkspaceVerifyResult> {
    let mut issues = Vec::new();

    // Check workspace.toml exists
    if !layout.workspace_manifest_path().is_file() {
        issues.push("workspace.toml not found — run `lexicon workspace init`".into());
        return Ok(WorkspaceVerifyResult {
            passed: false,
            issues,
        });
    }

    // Load and parse workspace manifest
    let content = fs::read_to_string(layout.workspace_manifest_path())?;
    let manifest: WorkspaceManifest = toml::from_str(&content)
        .map_err(|e| CoreError::Other(format!("Failed to parse workspace.toml: {e}")))?;

    // Check roles assigned
    if manifest.crate_roles.is_empty() {
        issues.push("No crate roles defined in workspace.toml".into());
    }

    // Check rules exist
    if !layout.architecture_rules_path().is_file() {
        issues.push("architecture/rules.toml not found".into());
    }

    // Check all workspace members have roles
    if let Ok(members) = parse_workspace_members(&layout.root) {
        let role_names: Vec<&str> = manifest.crate_roles.iter().map(|r| r.name.as_str()).collect();
        for member in &members {
            if !role_names.contains(&member.as_str()) {
                issues.push(format!("Crate '{}' has no assigned role", member));
            }
        }
    }

    let passed = issues.is_empty();
    Ok(WorkspaceVerifyResult { issues, passed })
}

/// Check workspace health and return a list of issues.
pub fn workspace_doctor(layout: &RepoLayout) -> CoreResult<Vec<String>> {
    let mut issues = Vec::new();

    // Check workspace.toml exists
    if !layout.workspace_manifest_path().is_file() {
        issues.push("workspace.toml not found — run `lexicon workspace init`".into());
        return Ok(issues);
    }

    // Check architecture rules exist
    if !layout.architecture_rules_path().is_file() {
        issues.push("architecture/rules.toml not found".into());
    }

    // Check architecture directory exists
    if !layout.architecture_dir().is_dir() {
        issues.push("architecture/ directory not found".into());
    }

    // Check all workspace members have assigned roles
    let content = fs::read_to_string(layout.workspace_manifest_path())?;
    let manifest: WorkspaceManifest = toml::from_str(&content)
        .map_err(|e| CoreError::Other(format!("Failed to parse workspace.toml: {e}")))?;

    if let Ok(members) = parse_workspace_members(&layout.root) {
        let role_names: Vec<&str> = manifest.crate_roles.iter().map(|r| r.name.as_str()).collect();
        for member in &members {
            if !role_names.contains(&member.as_str()) {
                issues.push(format!("Crate '{}' has no assigned role", member));
            }
        }
    }

    Ok(issues)
}

/// Parse workspace member crate names from Cargo.toml.
fn parse_workspace_members(root: &std::path::Path) -> CoreResult<Vec<String>> {
    let cargo_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)?;
    let doc: toml::Table = toml::from_str(&content)
        .map_err(|e| CoreError::Other(format!("Failed to parse Cargo.toml: {e}")))?;

    let members = doc
        .get("workspace")
        .and_then(|w| w.as_table())
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| {
                    // Extract crate name from path like "crates/foo"
                    s.rsplit('/').next().unwrap_or(s).to_string()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(members)
}

/// Infer a crate's architectural role from its name.
fn infer_role_from_name(name: &str) -> CrateRoleKind {
    let lower = name.to_lowercase();
    if lower.contains("cli") || lower.contains("bin") || lower.contains("app") {
        CrateRoleKind::Application
    } else if lower.contains("spec") || lower.contains("types") || lower.contains("model") {
        CrateRoleKind::Foundation
    } else if lower.contains("api") || lower.contains("trait") || lower.contains("interface") {
        CrateRoleKind::Interface
    } else if lower.contains("adapter") || lower.contains("driver") || lower.contains("plugin") {
        CrateRoleKind::Adapter
    } else if lower.contains("test") || lower.contains("fixture") || lower.contains("mock") {
        CrateRoleKind::Test
    } else if lower.contains("util") || lower.contains("common") || lower.contains("helper") || lower.contains("fs") {
        CrateRoleKind::Utility
    } else {
        CrateRoleKind::Foundation
    }
}

/// Return a sensible set of default dependency rules.
fn default_dependency_rules() -> Vec<DependencyRule> {
    vec![
        DependencyRule {
            from_role: CrateRoleKind::Foundation,
            allowed_targets: vec![CrateRoleKind::Utility],
            forbidden_targets: vec![CrateRoleKind::Application, CrateRoleKind::Adapter],
            description: "Foundation crates should only depend on utilities".into(),
        },
        DependencyRule {
            from_role: CrateRoleKind::Application,
            allowed_targets: vec![
                CrateRoleKind::Foundation,
                CrateRoleKind::Interface,
                CrateRoleKind::Adapter,
                CrateRoleKind::Utility,
            ],
            forbidden_targets: vec![CrateRoleKind::Test],
            description: "Application crates can depend on anything except test crates".into(),
        },
        DependencyRule {
            from_role: CrateRoleKind::Interface,
            allowed_targets: vec![CrateRoleKind::Foundation, CrateRoleKind::Utility],
            forbidden_targets: vec![
                CrateRoleKind::Application,
                CrateRoleKind::Adapter,
                CrateRoleKind::Test,
            ],
            description: "Interface crates should not depend on implementations".into(),
        },
        DependencyRule {
            from_role: CrateRoleKind::Adapter,
            allowed_targets: vec![
                CrateRoleKind::Foundation,
                CrateRoleKind::Interface,
                CrateRoleKind::Utility,
            ],
            forbidden_targets: vec![CrateRoleKind::Application, CrateRoleKind::Test],
            description: "Adapters implement interfaces; they should not depend on applications"
                .into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_workspace(tmp: &TempDir) {
        // Create Cargo.toml with workspace
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/my-cli\", \"crates/my-spec\", \"crates/my-util\"]\n",
        )
        .unwrap();

        // Create .lexicon directory (simulating init already run)
        fs::create_dir_all(tmp.path().join(".lexicon")).unwrap();
    }

    #[test]
    fn test_workspace_init() {
        let tmp = TempDir::new().unwrap();
        setup_workspace(&tmp);

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        workspace_init(&layout).unwrap();

        assert!(layout.workspace_manifest_path().is_file());
        assert!(layout.architecture_dir().is_dir());
        assert!(layout.architecture_rules_path().is_file());
    }

    #[test]
    fn test_workspace_init_requires_lexicon_dir() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        let result = workspace_init(&layout);
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_init_requires_workspace() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join(".lexicon")).unwrap();

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        let result = workspace_init(&layout);
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_verify_passes() {
        let tmp = TempDir::new().unwrap();
        setup_workspace(&tmp);

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        workspace_init(&layout).unwrap();

        let result = workspace_verify(&layout).unwrap();
        assert!(result.passed, "Expected verify to pass, issues: {:?}", result.issues);
    }

    #[test]
    fn test_workspace_doctor_healthy() {
        let tmp = TempDir::new().unwrap();
        setup_workspace(&tmp);

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        workspace_init(&layout).unwrap();

        let issues = workspace_doctor(&layout).unwrap();
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
    }

    #[test]
    fn test_infer_role_from_name() {
        assert_eq!(infer_role_from_name("my-cli"), CrateRoleKind::Application);
        assert_eq!(infer_role_from_name("my-spec"), CrateRoleKind::Foundation);
        assert_eq!(infer_role_from_name("my-api"), CrateRoleKind::Interface);
        assert_eq!(infer_role_from_name("my-adapter"), CrateRoleKind::Adapter);
        assert_eq!(infer_role_from_name("my-test"), CrateRoleKind::Test);
        assert_eq!(infer_role_from_name("my-util"), CrateRoleKind::Utility);
        assert_eq!(infer_role_from_name("my-fs"), CrateRoleKind::Utility);
        assert_eq!(infer_role_from_name("core"), CrateRoleKind::Foundation);
    }
}
