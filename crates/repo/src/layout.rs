use std::path::PathBuf;

/// Describes the directory layout for a lexicon-managed repository.
///
/// All paths are relative to the repository root.
#[derive(Debug, Clone)]
pub struct RepoLayout {
    /// Root of the repository.
    pub root: PathBuf,
}

impl RepoLayout {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Discover the repo root from the current working directory.
    ///
    /// Walks up from the current directory looking for a `.lexicon/` directory
    /// or a `Cargo.toml`. Falls back to the current directory if neither is found.
    pub fn discover() -> crate::error::RepoResult<Self> {
        let cwd = std::env::current_dir()?;
        let mut dir = cwd.as_path();

        loop {
            if dir.join(".lexicon").is_dir() || dir.join("Cargo.toml").is_file() {
                return Ok(Self::new(dir.to_path_buf()));
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => return Ok(Self::new(cwd)),
            }
        }
    }

    // --- .lexicon/ internal directories ---

    pub fn lexicon_dir(&self) -> PathBuf {
        self.root.join(".lexicon")
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.lexicon_dir().join("manifest.toml")
    }

    pub fn context_dir(&self) -> PathBuf {
        self.lexicon_dir().join("context")
    }

    pub fn conversations_dir(&self) -> PathBuf {
        self.lexicon_dir().join("conversations")
    }

    pub fn audit_dir(&self) -> PathBuf {
        self.lexicon_dir().join("audit")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.lexicon_dir().join("cache")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.lexicon_dir().join("state")
    }

    pub fn api_dir(&self) -> PathBuf {
        self.lexicon_dir().join("api")
    }

    pub fn auth_dir(&self) -> PathBuf {
        self.lexicon_dir().join("auth")
    }

    pub fn auth_credential_path(&self, provider: &str) -> PathBuf {
        self.auth_dir().join(format!("{provider}.json"))
    }

    // --- mode-aware paths ---

    pub fn workspace_manifest_path(&self) -> PathBuf {
        self.lexicon_dir().join("workspace.toml")
    }

    pub fn ecosystem_manifest_path(&self) -> PathBuf {
        self.lexicon_dir().join("ecosystem.toml")
    }

    pub fn architecture_dir(&self) -> PathBuf {
        self.lexicon_dir().join("architecture")
    }

    pub fn architecture_rules_path(&self) -> PathBuf {
        self.architecture_dir().join("rules.toml")
    }

    pub fn architecture_graph_path(&self) -> PathBuf {
        self.architecture_dir().join("graph.json")
    }

    pub fn ecosystem_dir(&self) -> PathBuf {
        self.lexicon_dir().join("ecosystem")
    }

    // --- specs/ directories ---

    pub fn specs_dir(&self) -> PathBuf {
        self.root.join("specs")
    }

    pub fn contracts_dir(&self) -> PathBuf {
        self.specs_dir().join("contracts")
    }

    pub fn behavior_dir(&self) -> PathBuf {
        self.specs_dir().join("behavior")
    }

    pub fn scoring_dir(&self) -> PathBuf {
        self.specs_dir().join("scoring")
    }

    pub fn non_goals_dir(&self) -> PathBuf {
        self.specs_dir().join("non_goals")
    }

    pub fn conformance_specs_dir(&self) -> PathBuf {
        self.specs_dir().join("conformance")
    }

    pub fn gates_path(&self) -> PathBuf {
        self.specs_dir().join("gates.toml")
    }

    pub fn scoring_model_path(&self) -> PathBuf {
        self.scoring_dir().join("model.toml")
    }

    pub fn prompts_dir(&self) -> PathBuf {
        self.specs_dir().join("prompts")
    }

    pub fn prompt_graph_path(&self) -> PathBuf {
        self.lexicon_dir().join("prompt-graph.json")
    }

    // --- test directories ---

    pub fn tests_dir(&self) -> PathBuf {
        self.root.join("tests")
    }

    pub fn conformance_tests_dir(&self) -> PathBuf {
        self.tests_dir().join("conformance")
    }

    pub fn behavior_tests_dir(&self) -> PathBuf {
        self.tests_dir().join("behavior")
    }

    pub fn integration_tests_dir(&self) -> PathBuf {
        self.tests_dir().join("integration")
    }

    pub fn property_tests_dir(&self) -> PathBuf {
        self.tests_dir().join("property")
    }

    pub fn edge_case_tests_dir(&self) -> PathBuf {
        self.tests_dir().join("edge_cases")
    }

    pub fn fuzz_targets_dir(&self) -> PathBuf {
        self.root.join("fuzz").join("fuzz_targets")
    }

    // --- other paths ---

    pub fn claude_md_path(&self) -> PathBuf {
        self.root.join("CLAUDE.md")
    }

    pub fn cargo_toml_path(&self) -> PathBuf {
        self.root.join("Cargo.toml")
    }

    /// All directories that should be created during `lexicon init`.
    pub fn init_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = vec![
            self.lexicon_dir(),
            self.context_dir(),
            self.conversations_dir(),
            self.audit_dir(),
            self.cache_dir(),
            self.state_dir(),
            self.api_dir(),
            self.auth_dir(),
            self.contracts_dir(),
            self.behavior_dir(),
            self.scoring_dir(),
            self.non_goals_dir(),
            self.conformance_specs_dir(),
            self.prompts_dir(),
        ];

        // Only create architecture/ and ecosystem/ dirs if .lexicon/ already exists.
        if self.lexicon_dir().is_dir() {
            dirs.push(self.architecture_dir());
            dirs.push(self.ecosystem_dir());
        }

        dirs
    }

    /// Returns true if lexicon has been initialized in this repo.
    pub fn is_initialized(&self) -> bool {
        self.manifest_path().exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_paths() {
        let layout = RepoLayout::new(PathBuf::from("/tmp/my-repo"));
        assert_eq!(layout.lexicon_dir(), PathBuf::from("/tmp/my-repo/.lexicon"));
        assert_eq!(
            layout.manifest_path(),
            PathBuf::from("/tmp/my-repo/.lexicon/manifest.toml")
        );
        assert_eq!(
            layout.contracts_dir(),
            PathBuf::from("/tmp/my-repo/specs/contracts")
        );
        assert_eq!(
            layout.gates_path(),
            PathBuf::from("/tmp/my-repo/specs/gates.toml")
        );
        assert_eq!(
            layout.claude_md_path(),
            PathBuf::from("/tmp/my-repo/CLAUDE.md")
        );
    }

    #[test]
    fn test_init_dirs_count() {
        let layout = RepoLayout::new(PathBuf::from("/tmp/test"));
        assert!(layout.init_dirs().len() >= 10);
    }

    #[test]
    fn test_mode_aware_paths() {
        let layout = RepoLayout::new(PathBuf::from("/tmp/my-repo"));
        assert_eq!(
            layout.workspace_manifest_path(),
            PathBuf::from("/tmp/my-repo/.lexicon/workspace.toml")
        );
        assert_eq!(
            layout.ecosystem_manifest_path(),
            PathBuf::from("/tmp/my-repo/.lexicon/ecosystem.toml")
        );
        assert_eq!(
            layout.architecture_dir(),
            PathBuf::from("/tmp/my-repo/.lexicon/architecture")
        );
        assert_eq!(
            layout.architecture_rules_path(),
            PathBuf::from("/tmp/my-repo/.lexicon/architecture/rules.toml")
        );
        assert_eq!(
            layout.architecture_graph_path(),
            PathBuf::from("/tmp/my-repo/.lexicon/architecture/graph.json")
        );
        assert_eq!(
            layout.ecosystem_dir(),
            PathBuf::from("/tmp/my-repo/.lexicon/ecosystem")
        );
    }
}
