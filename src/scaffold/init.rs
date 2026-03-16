use crate::repo::layout::RepoLayout;
use crate::repo::lexicon_dir;
use crate::spec::manifest::Manifest;

use super::error::{ScaffoldError, ScaffoldResult};

/// Initialize a repo with lexicon directory structure and manifest.
pub fn init_repo(layout: &RepoLayout, manifest: &Manifest) -> ScaffoldResult<()> {
    if layout.is_initialized() {
        return Err(ScaffoldError::AlreadyInitialized {
            path: layout.root.display().to_string(),
        });
    }

    // Create all directories
    lexicon_dir::ensure_lexicon_dirs(layout)?;

    // Write manifest
    lexicon_dir::save_manifest(layout, manifest)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::common::RepoType;
    use tempfile::TempDir;

    #[test]
    fn test_init_repo() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        let manifest = Manifest::new(
            "test".to_string(),
            "Test project".to_string(),
            RepoType::Library,
            "testing".to_string(),
        );
        init_repo(&layout, &manifest).unwrap();

        assert!(layout.manifest_path().exists());
        assert!(layout.contracts_dir().is_dir());
        assert!(layout.conversations_dir().is_dir());
        assert!(layout.audit_dir().is_dir());
    }

    #[test]
    fn test_init_already_initialized() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        let manifest = Manifest::new(
            "test".to_string(),
            "Test".to_string(),
            RepoType::Library,
            "testing".to_string(),
        );
        init_repo(&layout, &manifest).unwrap();

        // Second init should fail
        let result = init_repo(&layout, &manifest);
        assert!(result.is_err());
    }
}
