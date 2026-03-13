use serde::{Deserialize, Serialize};
use std::fmt;

/// Schema version for all lexicon artifacts.
///
/// Uses semantic versioning to track schema evolution
/// and enable migration-friendly artifact loading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
}

impl SchemaVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// The current schema version for all lexicon artifacts.
    pub const CURRENT: SchemaVersion = SchemaVersion::new(1, 0);

    /// Returns true if this version is compatible with `other`.
    /// Compatible means same major version and minor >= other's minor.
    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_display() {
        assert_eq!(SchemaVersion::new(1, 0).to_string(), "1.0");
        assert_eq!(SchemaVersion::new(2, 3).to_string(), "2.3");
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0 = SchemaVersion::new(1, 0);
        let v1_1 = SchemaVersion::new(1, 1);
        let v2_0 = SchemaVersion::new(2, 0);

        assert!(v1_0.is_compatible_with(&v1_0));
        assert!(v1_1.is_compatible_with(&v1_0));
        assert!(!v1_0.is_compatible_with(&v1_1));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_version_serde_roundtrip() {
        let v = SchemaVersion::new(1, 0);
        let json = serde_json::to_string(&v).unwrap();
        let parsed: SchemaVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(v, parsed);
    }
}
