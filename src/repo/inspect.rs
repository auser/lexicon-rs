use std::fs;
use std::path::Path;

use crate::spec::common::RepoType;

use super::error::{RepoError, RepoResult};

/// Information gathered from inspecting a repository.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    /// Name from Cargo.toml, if found.
    pub name: Option<String>,
    /// Description from Cargo.toml, if found.
    pub description: Option<String>,
    /// Detected repository type.
    pub repo_type: RepoType,
    /// Whether `src/lib.rs` exists.
    pub has_lib: bool,
    /// Whether `src/main.rs` exists.
    pub has_bin: bool,
    /// Whether this is a Cargo workspace.
    pub is_workspace: bool,
    /// Whether a `tests/` directory exists.
    pub has_tests_dir: bool,
    /// Whether a `benches/` directory exists.
    pub has_benches_dir: bool,
}

/// Inspect a repository root and gather information.
pub fn inspect_repo(root: &Path) -> RepoResult<RepoInfo> {
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(RepoError::NotARepo {
            path: root.display().to_string(),
        });
    }

    let cargo_content = fs::read_to_string(&cargo_toml)?;
    let cargo_value: toml::Value =
        toml::from_str(&cargo_content).map_err(|e| RepoError::Manifest(e.to_string()))?;

    let is_workspace = cargo_value.get("workspace").is_some();
    let has_lib = root.join("src/lib.rs").exists();
    let has_bin = root.join("src/main.rs").exists();
    let has_tests_dir = root.join("tests").is_dir();
    let has_benches_dir = root.join("benches").is_dir();

    let name = cargo_value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from);

    let description = cargo_value
        .get("package")
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str())
        .map(String::from);

    let repo_type = if is_workspace {
        RepoType::Workspace
    } else if has_lib {
        RepoType::Library
    } else {
        RepoType::Binary
    };

    Ok(RepoInfo {
        name,
        description,
        repo_type,
        has_lib,
        has_bin,
        is_workspace,
        has_tests_dir,
        has_benches_dir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_inspect_not_a_repo() {
        let dir = TempDir::new().unwrap();
        let result = inspect_repo(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_library() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"mylib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "").unwrap();

        let info = inspect_repo(dir.path()).unwrap();
        assert_eq!(info.name.as_deref(), Some("mylib"));
        assert_eq!(info.repo_type, RepoType::Library);
        assert!(info.has_lib);
        assert!(!info.has_bin);
    }

    #[test]
    fn test_inspect_workspace() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();

        let info = inspect_repo(dir.path()).unwrap();
        assert_eq!(info.repo_type, RepoType::Workspace);
        assert!(info.is_workspace);
    }
}
