//! Operating mode detection for progressive adoption.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The operating mode determines which capabilities are available.
///
/// Lexicon supports progressive adoption: start with a single repo,
/// expand to a workspace, then to an ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    /// Single repository mode (default).
    #[default]
    Repo,
    /// Multi-crate workspace mode.
    Workspace,
    /// Multi-repo ecosystem mode.
    Ecosystem,
}

impl fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repo => write!(f, "repo"),
            Self::Workspace => write!(f, "workspace"),
            Self::Ecosystem => write!(f, "ecosystem"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(OperatingMode::Repo.to_string(), "repo");
        assert_eq!(OperatingMode::Workspace.to_string(), "workspace");
        assert_eq!(OperatingMode::Ecosystem.to_string(), "ecosystem");
    }

    #[test]
    fn test_default_is_repo() {
        assert_eq!(OperatingMode::default(), OperatingMode::Repo);
    }

    #[test]
    fn test_serde_roundtrip() {
        for mode in [
            OperatingMode::Repo,
            OperatingMode::Workspace,
            OperatingMode::Ecosystem,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let back: OperatingMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, back);
        }
    }

    #[test]
    fn test_serde_values() {
        let json = serde_json::to_string(&OperatingMode::Repo).unwrap();
        assert_eq!(json, "\"repo\"");

        let json = serde_json::to_string(&OperatingMode::Workspace).unwrap();
        assert_eq!(json, "\"workspace\"");

        let json = serde_json::to_string(&OperatingMode::Ecosystem).unwrap();
        assert_eq!(json, "\"ecosystem\"");
    }
}
