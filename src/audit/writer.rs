use std::path::Path;

use crate::spec::audit::AuditRecord;

use super::error::AuditResult;

/// Write an audit record to the audit directory.
///
/// File naming convention: `<timestamp>-<uuid>.json`
pub fn write_audit_record(audit_dir: &Path, record: &AuditRecord) -> AuditResult<()> {
    let timestamp = record.timestamp.format("%Y%m%dT%H%M%S");
    let filename = format!("{}-{}.json", timestamp, record.id);
    let path = audit_dir.join(filename);
    let content = serde_json::to_string_pretty(record)?;
    crate::fs::safe_write::safe_write(&path, &content, false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::audit::AuditRecord;
    use crate::spec::common::{Actor, AuditAction};
    use tempfile::TempDir;

    #[test]
    fn test_write_audit_record() {
        let dir = TempDir::new().unwrap();
        let record = AuditRecord::new(
            AuditAction::RepoInit,
            Actor::User,
            "Initialized repository".to_string(),
        );
        write_audit_record(dir.path(), &record).unwrap();

        // Verify file was created
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);

        // Verify content
        let content = std::fs::read_to_string(entries[0].path()).unwrap();
        let parsed: AuditRecord = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.id, record.id);
        assert_eq!(parsed.action, AuditAction::RepoInit);
    }
}
