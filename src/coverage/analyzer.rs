//! Scan test files for lexicon tags.
//!
//! Recognizes three tag formats:
//! - `#[lexicon_tag("tag-name")]` attribute on test functions
//! - `lexicon::tags("tag1", "tag2")` function calls in test bodies
//! - `// lexicon-tag: tag-name` comments

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::CoverageError;

/// A tag found in a test file, linking a test to a contract clause.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestTag {
    /// The tag value (e.g. "conformance", "safety").
    pub tag: String,
    /// The name of the test function this tag is associated with.
    pub test_name: String,
    /// The file path where the tag was found.
    pub file_path: String,
    /// The line number where the tag was found (1-based).
    pub line: u32,
}

/// Scan a single Rust source file for lexicon tags.
pub fn scan_file(path: &Path) -> Result<Vec<TestTag>, CoverageError> {
    let content = fs::read_to_string(path)?;
    let file_path = path.display().to_string();
    let mut tags = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    // Track the current test function name as we scan.
    let mut current_test_fn: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
        let line_num = (i + 1) as u32;
        let trimmed = line.trim();

        // Detect test function declarations to track context.
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
            if let Some(name) = extract_fn_name(trimmed) {
                current_test_fn = Some(name);
            }
        }

        // Pattern 1: #[lexicon_tag("tag-name")]
        if trimmed.starts_with("#[lexicon_tag(") {
            if let Some(tag) = extract_attribute_tag(trimmed) {
                // Look ahead for the function name.
                let test_name = find_next_fn_name(&lines, i)
                    .unwrap_or_else(|| "unknown".to_string());
                tags.push(TestTag {
                    tag,
                    test_name,
                    file_path: file_path.clone(),
                    line: line_num,
                });
            }
        }

        // Pattern 2: lexicon::tags("tag1", "tag2")
        if trimmed.contains("lexicon::tags(") {
            let extracted = extract_function_tags(trimmed);
            let test_name = current_test_fn
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            for tag in extracted {
                tags.push(TestTag {
                    tag,
                    test_name: test_name.clone(),
                    file_path: file_path.clone(),
                    line: line_num,
                });
            }
        }

        // Pattern 3: // lexicon-tag: tag-name
        if trimmed.starts_with("// lexicon-tag:") {
            if let Some(tag) = trimmed.strip_prefix("// lexicon-tag:") {
                let tag = tag.trim().to_string();
                if !tag.is_empty() {
                    let test_name = current_test_fn
                        .clone()
                        .or_else(|| find_next_fn_name(&lines, i))
                        .unwrap_or_else(|| "unknown".to_string());
                    tags.push(TestTag {
                        tag,
                        test_name,
                        file_path: file_path.clone(),
                        line: line_num,
                    });
                }
            }
        }
    }

    Ok(tags)
}

/// Scan a directory recursively for `.rs` files and collect all tags.
pub fn scan_directory(dir: &Path) -> Result<Vec<TestTag>, CoverageError> {
    let pattern = format!("{}/**/*.rs", dir.display());
    let mut all_tags = Vec::new();

    let paths = glob::glob(&pattern).map_err(|e| CoverageError::Parse(e.to_string()))?;

    for entry in paths {
        let path = entry.map_err(|e| CoverageError::Io(e.into_error()))?;
        let file_tags = scan_file(&path)?;
        all_tags.extend(file_tags);
    }

    Ok(all_tags)
}

/// Extract the tag value from `#[lexicon_tag("tag-name")]`.
fn extract_attribute_tag(line: &str) -> Option<String> {
    let start = line.find("lexicon_tag(\"")? + "lexicon_tag(\"".len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Extract all tag values from `lexicon::tags("tag1", "tag2")`.
fn extract_function_tags(line: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let Some(start) = line.find("lexicon::tags(") else {
        return tags;
    };
    let rest = &line[start + "lexicon::tags(".len()..];
    // Find all quoted strings within the parentheses.
    let mut chars = rest.chars();
    let mut in_quote = false;
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            ')' if !in_quote => break,
            '"' => {
                if in_quote {
                    tags.push(current.clone());
                    current.clear();
                }
                in_quote = !in_quote;
            }
            _ if in_quote => current.push(ch),
            _ => {}
        }
    }

    tags
}

/// Extract a function name from a `fn` declaration line.
fn extract_fn_name(line: &str) -> Option<String> {
    // Handle: fn name(...), pub fn name(...), async fn name(...), pub async fn name(...)
    let line = line.trim();
    let after_fn = if let Some(rest) = line.strip_prefix("pub async fn ") {
        rest
    } else if let Some(rest) = line.strip_prefix("async fn ") {
        rest
    } else if let Some(rest) = line.strip_prefix("pub fn ") {
        rest
    } else if let Some(rest) = line.strip_prefix("fn ") {
        rest
    } else {
        return None;
    };

    let name_end = after_fn.find(|c: char| c == '(' || c == '<' || c.is_whitespace())?;
    Some(after_fn[..name_end].to_string())
}

/// Look ahead from the current position to find the next function name.
fn find_next_fn_name(lines: &[&str], from: usize) -> Option<String> {
    for line in &lines[from + 1..] {
        let trimmed = line.trim();
        if let Some(name) = extract_fn_name(trimmed) {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_scan_attribute_tag() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
#[lexicon_tag("conformance")]
#[test]
fn test_get_returns_none() {{
    // test body
}}
"#
        )
        .unwrap();

        let tags = scan_file(f.path()).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag, "conformance");
        assert_eq!(tags[0].test_name, "test_get_returns_none");
    }

    #[test]
    fn test_scan_function_tags() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
#[test]
fn test_safety_check() {{
    lexicon::tags("safety", "edge-case");
    // test body
}}
"#
        )
        .unwrap();

        let tags = scan_file(f.path()).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag, "safety");
        assert_eq!(tags[1].tag, "edge-case");
        assert_eq!(tags[0].test_name, "test_safety_check");
    }

    #[test]
    fn test_scan_comment_tag() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
// lexicon-tag: basic
#[test]
fn test_basic_insert() {{
    // test body
}}
"#
        )
        .unwrap();

        let tags = scan_file(f.path()).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag, "basic");
        assert_eq!(tags[0].test_name, "test_basic_insert");
    }

    #[test]
    fn test_scan_multiple_formats() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
#[lexicon_tag("conformance")]
#[test]
fn test_one() {{
    lexicon::tags("extra-tag");
}}

// lexicon-tag: safety
#[test]
fn test_two() {{}}
"#
        )
        .unwrap();

        let tags = scan_file(f.path()).unwrap();
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_scan_directory() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_file.rs");
        fs::write(
            &file_path,
            r#"
#[lexicon_tag("conformance")]
#[test]
fn test_example() {}
"#,
        )
        .unwrap();

        let tags = scan_directory(dir.path()).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag, "conformance");
    }

    #[test]
    fn test_no_tags_found() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "fn main() {{ println!(\"hello\"); }}").unwrap();

        let tags = scan_file(f.path()).unwrap();
        assert!(tags.is_empty());
    }
}
