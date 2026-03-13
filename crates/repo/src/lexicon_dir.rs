use std::fs;
use std::path::Path;

use lexicon_spec::manifest::Manifest;

use crate::error::{RepoError, RepoResult};
use crate::layout::RepoLayout;

/// Ensure the `.lexicon/` directory and all subdirectories exist.
pub fn ensure_lexicon_dirs(layout: &RepoLayout) -> RepoResult<()> {
    for dir in layout.init_dirs() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Load the manifest from `.lexicon/manifest.toml`.
pub fn load_manifest(layout: &RepoLayout) -> RepoResult<Manifest> {
    let path = layout.manifest_path();
    if !path.exists() {
        return Err(RepoError::NotInitialized {
            path: layout.root.display().to_string(),
        });
    }
    let content = fs::read_to_string(&path)?;
    let manifest: Manifest = toml::from_str(&content)?;
    Ok(manifest)
}

/// Save the manifest to `.lexicon/manifest.toml`.
pub fn save_manifest(layout: &RepoLayout, manifest: &Manifest) -> RepoResult<()> {
    let content = toml::to_string_pretty(manifest)?;
    let path = layout.manifest_path();
    lexicon_fs::safe_write::safe_write(&path, &content, false)?;
    Ok(())
}

/// Check if a directory looks like a valid repo root (has Cargo.toml).
pub fn validate_repo_root(path: &Path) -> RepoResult<()> {
    if !path.join("Cargo.toml").exists() {
        return Err(RepoError::NotARepo {
            path: path.display().to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::common::RepoType;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_lexicon_dirs() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        ensure_lexicon_dirs(&layout).unwrap();
        assert!(layout.lexicon_dir().is_dir());
        assert!(layout.context_dir().is_dir());
        assert!(layout.conversations_dir().is_dir());
        assert!(layout.audit_dir().is_dir());
        assert!(layout.contracts_dir().is_dir());
    }

    #[test]
    fn test_save_and_load_manifest() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        ensure_lexicon_dirs(&layout).unwrap();

        let manifest = Manifest::new(
            "test-project".to_string(),
            "A test project".to_string(),
            RepoType::Library,
            "testing".to_string(),
        );
        save_manifest(&layout, &manifest).unwrap();

        let loaded = load_manifest(&layout).unwrap();
        assert_eq!(loaded.project.name, "test-project");
    }

    #[test]
    fn test_load_manifest_not_initialized() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        let result = load_manifest(&layout);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_repo_root() {
        let dir = TempDir::new().unwrap();
        assert!(validate_repo_root(dir.path()).is_err());

        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        assert!(validate_repo_root(dir.path()).is_ok());
    }
}
