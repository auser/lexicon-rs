//! Capability model for progressive adoption.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::mode::OperatingMode;

/// A discrete capability that may be enabled depending on the operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // --- Repo-level ---
    RepoContracts,
    RepoConformance,
    RepoScoring,
    RepoGates,
    RepoApi,

    // --- Workspace-level ---
    WorkspaceArchitecture,
    WorkspaceDependencyLaw,
    WorkspaceSharedContracts,

    // --- Ecosystem-level ---
    EcosystemGovernance,
    EcosystemSharedContracts,
    EcosystemImpact,
}

/// A set of capabilities enabled for a given operating mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    inner: HashSet<Capability>,
}

impl CapabilitySet {
    /// Build the capability set for the given mode.
    pub fn for_mode(mode: OperatingMode) -> Self {
        let mut caps = HashSet::new();

        // Repo capabilities are always present.
        caps.insert(Capability::RepoContracts);
        caps.insert(Capability::RepoConformance);
        caps.insert(Capability::RepoScoring);
        caps.insert(Capability::RepoGates);
        caps.insert(Capability::RepoApi);

        if matches!(mode, OperatingMode::Workspace | OperatingMode::Ecosystem) {
            caps.insert(Capability::WorkspaceArchitecture);
            caps.insert(Capability::WorkspaceDependencyLaw);
            caps.insert(Capability::WorkspaceSharedContracts);
        }

        if mode == OperatingMode::Ecosystem {
            caps.insert(Capability::EcosystemGovernance);
            caps.insert(Capability::EcosystemSharedContracts);
            caps.insert(Capability::EcosystemImpact);
        }

        Self { inner: caps }
    }

    /// Check whether the set contains a capability.
    pub fn has(&self, cap: Capability) -> bool {
        self.inner.contains(&cap)
    }

    /// Returns true if no capabilities are enabled.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_capabilities() {
        let set = CapabilitySet::for_mode(OperatingMode::Repo);
        assert_eq!(set.inner.len(), 5);
        assert!(set.has(Capability::RepoContracts));
        assert!(set.has(Capability::RepoConformance));
        assert!(set.has(Capability::RepoScoring));
        assert!(set.has(Capability::RepoGates));
        assert!(set.has(Capability::RepoApi));
        assert!(!set.has(Capability::WorkspaceArchitecture));
        assert!(!set.has(Capability::EcosystemGovernance));
    }

    #[test]
    fn test_workspace_capabilities() {
        let set = CapabilitySet::for_mode(OperatingMode::Workspace);
        assert_eq!(set.inner.len(), 8);
        // Has all repo caps
        assert!(set.has(Capability::RepoContracts));
        assert!(set.has(Capability::RepoApi));
        // Plus workspace caps
        assert!(set.has(Capability::WorkspaceArchitecture));
        assert!(set.has(Capability::WorkspaceDependencyLaw));
        assert!(set.has(Capability::WorkspaceSharedContracts));
        // But not ecosystem
        assert!(!set.has(Capability::EcosystemGovernance));
    }

    #[test]
    fn test_ecosystem_capabilities() {
        let set = CapabilitySet::for_mode(OperatingMode::Ecosystem);
        assert_eq!(set.inner.len(), 11);
        assert!(set.has(Capability::RepoContracts));
        assert!(set.has(Capability::WorkspaceArchitecture));
        assert!(set.has(Capability::EcosystemGovernance));
        assert!(set.has(Capability::EcosystemSharedContracts));
        assert!(set.has(Capability::EcosystemImpact));
    }

    #[test]
    fn test_is_empty() {
        let set = CapabilitySet::for_mode(OperatingMode::Repo);
        assert!(!set.is_empty());
    }

    #[test]
    fn test_capability_serde_roundtrip() {
        let cap = Capability::WorkspaceArchitecture;
        let json = serde_json::to_string(&cap).unwrap();
        let back: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, back);
    }
}
