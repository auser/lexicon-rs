use std::fs;
use std::path::Path;

use crate::error::{FsError, FsResult};

/// Atomically write content to a file.
///
/// Writes to a temporary file first, then renames to the target path.
/// If `backup` is true and the target exists, a `.bak` copy is created first.
pub fn safe_write(path: &Path, content: &str, backup: bool) -> FsResult<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create backup if requested and file exists
    if backup && path.exists() {
        let backup_path = path.with_extension("bak");
        fs::copy(path, &backup_path).map_err(|e| FsError::BackupFailed {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;
    }

    // Write to temp file in same directory, then rename for atomicity
    let dir = path.parent().unwrap_or(Path::new("."));
    let temp = tempfile::NamedTempFile::new_in(dir)?;
    fs::write(temp.path(), content)?;
    temp.persist(path).map_err(|e| {
        FsError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })?;

    Ok(())
}

/// Ensure a directory exists, creating it and parents if needed.
pub fn ensure_dir(path: &Path) -> FsResult<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_safe_write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        safe_write(&path, "hello", false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn test_safe_write_overwrites() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        safe_write(&path, "first", false).unwrap();
        safe_write(&path, "second", false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
    }

    #[test]
    fn test_safe_write_with_backup() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        safe_write(&path, "original", false).unwrap();
        safe_write(&path, "updated", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "updated");
        assert_eq!(
            fs::read_to_string(path.with_extension("bak")).unwrap(),
            "original"
        );
    }

    #[test]
    fn test_safe_write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("test.txt");
        safe_write(&path, "nested content", false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "nested content");
    }

    #[test]
    fn test_ensure_dir() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a").join("b").join("c");
        ensure_dir(&path).unwrap();
        assert!(path.is_dir());
    }
}
