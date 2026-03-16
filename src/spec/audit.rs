use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Actor, AuditAction};
use super::version::SchemaVersion;

/// An audit record tracking a significant action.
///
/// Audit records provide an immutable trail of changes, especially
/// for AI-guided modifications. They record what changed, who did it,
/// and what the impact was on scoring and gates.
///
/// Stored at `.lexicon/audit/<timestamp>-<uuid>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub schema_version: SchemaVersion,
    pub id: Uuid,
    pub action: AuditAction,
    pub actor: Actor,
    /// Content hash before the change.
    #[serde(default)]
    pub before_hash: Option<String>,
    /// Content hash after the change.
    #[serde(default)]
    pub after_hash: Option<String>,
    /// Human-readable summary of what changed.
    pub delta_summary: String,
    /// Score before the change, if applicable.
    #[serde(default)]
    pub score_before: Option<f64>,
    /// Score after the change, if applicable.
    #[serde(default)]
    pub score_after: Option<f64>,
    /// Whether all gates passed after the change.
    pub gates_passed: bool,
    pub timestamp: DateTime<Utc>,
}

impl AuditRecord {
    /// Create a new audit record.
    pub fn new(action: AuditAction, actor: Actor, delta_summary: String) -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            id: Uuid::new_v4(),
            action,
            actor,
            before_hash: None,
            after_hash: None,
            delta_summary,
            score_before: None,
            score_after: None,
            gates_passed: true,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_record_json_roundtrip() {
        let record = AuditRecord::new(
            AuditAction::ContractCreate,
            Actor::User,
            "Created contract key-value-store".to_string(),
        );
        let json = serde_json::to_string_pretty(&record).unwrap();
        let parsed: AuditRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, record.id);
        assert_eq!(parsed.action, AuditAction::ContractCreate);
        assert_eq!(parsed.actor, Actor::User);
    }
}
