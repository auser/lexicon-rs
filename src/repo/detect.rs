//! Repository shape detection and operating mode inference.

use std::path::Path;

use crate::spec::mode::OperatingMode;

use super::layout::RepoLayout;

/// The detected shape of a repository on disk.
#[derive(Debug, Clone, PartialEq)]
pub enum RepoShape {
    /// A single-crate repository (one Cargo.toml, no [workspace]).
    SingleCrate,
    /// A Cargo workspace with multiple members.
    Workspace {
        /// Number of workspace members detected.
        member_count: usize,
    },
    /// An ecosystem root (has `.lexicon/ecosystem.toml`).
    Ecosystem,
}

/// Detect the shape of a repository from its root path.
///
/// Checks for:
/// 1. `.lexicon/ecosystem.toml` -> Ecosystem
/// 2. `Cargo.toml` with `[workspace]` section -> Workspace
/// 3. `Cargo.toml` without `[workspace]` -> SingleCrate
/// 4. No `Cargo.toml` -> SingleCrate (fallback)
pub fn detect_shape(root: &Path) -> RepoShape {
    // Check for ecosystem marker first.
    if root.join(".lexicon/ecosystem.toml").is_file() {
        return RepoShape::Ecosystem;
    }

    // Check Cargo.toml for workspace section.
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.is_file() {
        if let Ok(contents) = std::fs::read_to_string(&cargo_path) {
            if let Ok(doc) = contents.parse::<toml::Table>() {
                if let Some(workspace) = doc.get("workspace").and_then(|v| v.as_table()) {
                    let member_count = workspace
                        .get("members")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    return RepoShape::Workspace { member_count };
                }
            }
        }
    }

    RepoShape::SingleCrate
}

/// Detect the operating mode from a repository layout.
///
/// Uses file-system signals:
/// - `.lexicon/ecosystem.toml` exists -> Ecosystem
/// - `.lexicon/workspace.toml` exists or Cargo workspace detected -> Workspace
/// - Otherwise -> Repo
pub fn detect_mode(layout: &RepoLayout) -> OperatingMode {
    if layout.ecosystem_manifest_path().is_file() {
        return OperatingMode::Ecosystem;
    }

    if layout.workspace_manifest_path().is_file() {
        return OperatingMode::Workspace;
    }

    // Also promote to Workspace if the Cargo.toml has a [workspace] section.
    let cargo_path = layout.cargo_toml_path();
    if cargo_path.is_file() {
        if let Ok(contents) = std::fs::read_to_string(&cargo_path) {
            if let Ok(doc) = contents.parse::<toml::Table>() {
                if doc.contains_key("workspace") {
                    return OperatingMode::Workspace;
                }
            }
        }
    }

    OperatingMode::Repo
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_shape_single_crate() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"my-crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        assert_eq!(detect_shape(tmp.path()), RepoShape::SingleCrate);
    }

    #[test]
    fn test_detect_shape_workspace() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crate-a\", \"crate-b\", \"crate-c\"]\n",
        )
        .unwrap();

        assert_eq!(
            detect_shape(tmp.path()),
            RepoShape::Workspace { member_count: 3 }
        );
    }

    #[test]
    fn test_detect_shape_ecosystem() {
        let tmp = TempDir::new().unwrap();
        let lexicon_dir = tmp.path().join(".lexicon");
        fs::create_dir_all(&lexicon_dir).unwrap();
        fs::write(
            lexicon_dir.join("ecosystem.toml"),
            "name = \"my-ecosystem\"\n",
        )
        .unwrap();

        assert_eq!(detect_shape(tmp.path()), RepoShape::Ecosystem);
    }

    #[test]
    fn test_detect_shape_no_cargo() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_shape(tmp.path()), RepoShape::SingleCrate);
    }

    #[test]
    fn test_detect_mode_repo() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());
        assert_eq!(detect_mode(&layout), OperatingMode::Repo);
    }

    #[test]
    fn test_detect_mode_workspace_from_toml() {
        let tmp = TempDir::new().unwrap();
        let lexicon_dir = tmp.path().join(".lexicon");
        fs::create_dir_all(&lexicon_dir).unwrap();
        fs::write(
            lexicon_dir.join("workspace.toml"),
            "mode = \"workspace\"\n",
        )
        .unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());
        assert_eq!(detect_mode(&layout), OperatingMode::Workspace);
    }

    #[test]
    fn test_detect_mode_workspace_from_cargo() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"a\"]\n",
        )
        .unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());
        assert_eq!(detect_mode(&layout), OperatingMode::Workspace);
    }

    #[test]
    fn test_detect_mode_ecosystem() {
        let tmp = TempDir::new().unwrap();
        let lexicon_dir = tmp.path().join(".lexicon");
        fs::create_dir_all(&lexicon_dir).unwrap();
        fs::write(
            lexicon_dir.join("ecosystem.toml"),
            "name = \"eco\"\n",
        )
        .unwrap();
        let layout = RepoLayout::new(tmp.path().to_path_buf());
        assert_eq!(detect_mode(&layout), OperatingMode::Ecosystem);
    }
}
