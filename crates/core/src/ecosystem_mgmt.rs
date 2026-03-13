//! Ecosystem governance initialization and health checks.

use std::fs;

use lexicon_repo::layout::RepoLayout;
use lexicon_spec::ecosystem::EcosystemManifest;

use crate::error::{CoreError, CoreResult};

/// Result of ecosystem verification.
#[derive(Debug)]
pub struct EcosystemVerifyResult {
    pub issues: Vec<String>,
    pub passed: bool,
}

/// Initialize ecosystem governance.
///
/// Requires that `lexicon init` has already been run.
pub fn ecosystem_init(layout: &RepoLayout) -> CoreResult<()> {
    // Verify .lexicon/ exists
    if !layout.lexicon_dir().is_dir() {
        return Err(CoreError::Other(
            "No .lexicon/ directory found. Run `lexicon init` first.".into(),
        ));
    }

    // Infer a default ecosystem name from the directory
    let name = layout
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-ecosystem")
        .to_string();

    // Create a default ecosystem manifest
    let manifest = EcosystemManifest {
        name,
        repos: Vec::new(),
        shared_contracts: Vec::new(),
        governance_rules: Vec::new(),
    };

    // Write ecosystem.toml
    let toml_str = toml::to_string_pretty(&manifest)
        .map_err(|e| CoreError::Other(format!("Failed to serialize ecosystem manifest: {e}")))?;
    fs::write(layout.ecosystem_manifest_path(), toml_str)?;

    // Create ecosystem directory
    fs::create_dir_all(layout.ecosystem_dir())?;

    Ok(())
}

/// Verify ecosystem governance.
pub fn ecosystem_verify(layout: &RepoLayout) -> CoreResult<EcosystemVerifyResult> {
    let mut issues = Vec::new();

    // Check ecosystem.toml exists
    if !layout.ecosystem_manifest_path().is_file() {
        issues.push("ecosystem.toml not found — run `lexicon ecosystem init`".into());
        return Ok(EcosystemVerifyResult {
            passed: false,
            issues,
        });
    }

    // Load and parse ecosystem manifest
    let content = fs::read_to_string(layout.ecosystem_manifest_path())?;
    let manifest: EcosystemManifest = toml::from_str(&content)
        .map_err(|e| CoreError::Other(format!("Failed to parse ecosystem.toml: {e}")))?;

    // Check repos listed
    if manifest.repos.is_empty() {
        issues.push("No repositories listed in ecosystem.toml".into());
    }

    let passed = issues.is_empty();
    Ok(EcosystemVerifyResult { issues, passed })
}

/// Check ecosystem health and return a list of issues.
pub fn ecosystem_doctor(layout: &RepoLayout) -> CoreResult<Vec<String>> {
    let mut issues = Vec::new();

    // Check ecosystem.toml exists
    if !layout.ecosystem_manifest_path().is_file() {
        issues.push("ecosystem.toml not found — run `lexicon ecosystem init`".into());
    }

    // Check ecosystem directory exists
    if !layout.ecosystem_dir().is_dir() {
        issues.push("ecosystem/ directory not found".into());
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_initialized(tmp: &TempDir) {
        fs::create_dir_all(tmp.path().join(".lexicon")).unwrap();
    }

    #[test]
    fn test_ecosystem_init() {
        let tmp = TempDir::new().unwrap();
        setup_initialized(&tmp);

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        ecosystem_init(&layout).unwrap();

        assert!(layout.ecosystem_manifest_path().is_file());
        assert!(layout.ecosystem_dir().is_dir());
    }

    #[test]
    fn test_ecosystem_init_requires_lexicon_dir() {
        let tmp = TempDir::new().unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());
        let result = ecosystem_init(&layout);
        assert!(result.is_err());
    }

    #[test]
    fn test_ecosystem_verify_no_manifest() {
        let tmp = TempDir::new().unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());

        let result = ecosystem_verify(&layout).unwrap();
        assert!(!result.passed);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_ecosystem_verify_empty_repos() {
        let tmp = TempDir::new().unwrap();
        setup_initialized(&tmp);

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        ecosystem_init(&layout).unwrap();

        let result = ecosystem_verify(&layout).unwrap();
        // Should report that no repos are listed
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("No repositories")));
    }

    #[test]
    fn test_ecosystem_doctor_healthy() {
        let tmp = TempDir::new().unwrap();
        setup_initialized(&tmp);

        let layout = RepoLayout::new(tmp.path().to_path_buf());
        ecosystem_init(&layout).unwrap();

        let issues = ecosystem_doctor(&layout).unwrap();
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
    }

    #[test]
    fn test_ecosystem_doctor_missing() {
        let tmp = TempDir::new().unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());

        let issues = ecosystem_doctor(&layout).unwrap();
        assert!(!issues.is_empty());
    }
}
