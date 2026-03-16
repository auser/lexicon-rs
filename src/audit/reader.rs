use std::fs;
use std::path::Path;

use crate::spec::audit::AuditRecord;
use crate::spec::common::AuditAction;

use crate::audit::error::AuditResult;

/// List all audit records in the audit directory, sorted by timestamp.
pub fn list_audit_records(audit_dir: &Path) -> AuditResult<Vec<AuditRecord>> {
    if !audit_dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(audit_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .collect();

    // Sort by filename (which includes timestamp prefix)
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let content = fs::read_to_string(entry.path())?;
        let record: AuditRecord = serde_json::from_str(&content)?;
        records.push(record);
    }

    Ok(records)
}

/// Filter audit records by action type.
pub fn filter_by_action(records: &[AuditRecord], action: AuditAction) -> Vec<&AuditRecord> {
    records.iter().filter(|r| r.action == action).collect()
}

/// Get the most recent audit record, if any.
pub fn latest_record(audit_dir: &Path) -> AuditResult<Option<AuditRecord>> {
    let records = list_audit_records(audit_dir)?;
    Ok(records.into_iter().last())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::writer::write_audit_record;
    use crate::spec::audit::AuditRecord;
    use crate::spec::common::Actor;
    use tempfile::TempDir;

    #[test]
    fn test_list_empty_dir() {
        let dir = TempDir::new().unwrap();
        let records = list_audit_records(dir.path()).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_list_nonexistent_dir() {
        let records = list_audit_records(Path::new("/nonexistent")).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_list_and_filter() {
        let dir = TempDir::new().unwrap();

        let r1 = AuditRecord::new(AuditAction::RepoInit, Actor::User, "init".to_string());
        let r2 = AuditRecord::new(
            AuditAction::ContractCreate,
            Actor::User,
            "created contract".to_string(),
        );
        write_audit_record(dir.path(), &r1).unwrap();
        write_audit_record(dir.path(), &r2).unwrap();

        let records = list_audit_records(dir.path()).unwrap();
        assert_eq!(records.len(), 2);

        let init_records = filter_by_action(&records, AuditAction::RepoInit);
        assert_eq!(init_records.len(), 1);
    }
}
