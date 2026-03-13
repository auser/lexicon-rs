use std::path::Path;

use crate::error::ApiError;
use crate::schema::ApiSnapshot;

/// Save an API snapshot as a baseline JSON file.
pub fn save_baseline(snapshot: &ApiSnapshot, path: &Path) -> Result<(), ApiError> {
    let json = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load an API snapshot baseline from a JSON file.
pub fn load_baseline(path: &Path) -> Result<ApiSnapshot, ApiError> {
    if !path.exists() {
        return Err(ApiError::BaselineNotFound(
            path.to_string_lossy().to_string(),
        ));
    }
    let contents = std::fs::read_to_string(path)?;
    let snapshot: ApiSnapshot = serde_json::from_str(&contents)?;
    Ok(snapshot)
}

/// Save the current API snapshot as current.json in the given directory.
pub fn save_current(snapshot: &ApiSnapshot, path: &Path) -> Result<(), ApiError> {
    save_baseline(snapshot, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ApiItem, ApiItemKind, Visibility};

    fn sample_snapshot() -> ApiSnapshot {
        ApiSnapshot {
            crate_name: "test-crate".into(),
            version: Some("0.1.0".into()),
            items: vec![ApiItem {
                kind: ApiItemKind::Function,
                name: "hello".into(),
                module_path: vec![],
                signature: "fn hello()".into(),
                visibility: Visibility::Public,
                trait_associations: vec![],
                stability: None,
                doc_summary: None,
                span_file: None,
                span_line: None,
            }],
            extracted_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("baseline.json");
        let snap = sample_snapshot();
        save_baseline(&snap, &path).unwrap();
        let loaded = load_baseline(&path).unwrap();
        assert_eq!(loaded.crate_name, snap.crate_name);
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0], snap.items[0]);
    }

    #[test]
    fn load_missing_baseline() {
        let result = load_baseline(Path::new("/nonexistent/baseline.json"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::BaselineNotFound(_)));
    }

    #[test]
    fn save_current_works() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("current.json");
        let snap = sample_snapshot();
        save_current(&snap, &path).unwrap();
        let loaded = load_baseline(&path).unwrap();
        assert_eq!(loaded.crate_name, snap.crate_name);
    }
}
