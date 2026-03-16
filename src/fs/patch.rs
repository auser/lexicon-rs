use super::error::{FsError, FsResult};

/// Marker format for managed blocks.
///
/// Managed blocks are delimited by start and end markers:
/// ```text
/// <!-- lexicon:begin:MARKER -->
/// managed content here
/// <!-- lexicon:end:MARKER -->
/// ```
const BEGIN_PREFIX: &str = "<!-- lexicon:begin:";
const END_PREFIX: &str = "<!-- lexicon:end:";
const MARKER_SUFFIX: &str = " -->";

/// Insert or update a managed block in content.
///
/// If a block with the given marker exists, its content is replaced.
/// If not, the block is appended to the end of the content.
pub fn upsert_managed_block(content: &str, marker: &str, block_content: &str) -> FsResult<String> {
    let begin_tag = format!("{BEGIN_PREFIX}{marker}{MARKER_SUFFIX}");
    let end_tag = format!("{END_PREFIX}{marker}{MARKER_SUFFIX}");

    if let Some(begin_pos) = content.find(&begin_tag) {
        // Update existing block
        let after_begin = begin_pos + begin_tag.len();
        let end_pos = content[after_begin..]
            .find(&end_tag)
            .ok_or_else(|| FsError::MalformedManagedBlock {
                path: format!("(inline content, marker={marker})"),
            })?;
        let end_pos = after_begin + end_pos;

        let mut result = String::with_capacity(content.len());
        result.push_str(&content[..after_begin]);
        result.push('\n');
        result.push_str(block_content);
        if !block_content.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&content[end_pos..]);
        Ok(result)
    } else {
        // Append new block
        let mut result = content.to_string();
        if !result.ends_with('\n') && !result.is_empty() {
            result.push('\n');
        }
        result.push('\n');
        result.push_str(&begin_tag);
        result.push('\n');
        result.push_str(block_content);
        if !block_content.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&end_tag);
        result.push('\n');
        Ok(result)
    }
}

/// Extract the content of a managed block.
pub fn extract_managed_block(content: &str, marker: &str) -> FsResult<Option<String>> {
    let begin_tag = format!("{BEGIN_PREFIX}{marker}{MARKER_SUFFIX}");
    let end_tag = format!("{END_PREFIX}{marker}{MARKER_SUFFIX}");

    if let Some(begin_pos) = content.find(&begin_tag) {
        let after_begin = begin_pos + begin_tag.len();
        let end_pos = content[after_begin..]
            .find(&end_tag)
            .ok_or_else(|| FsError::MalformedManagedBlock {
                path: format!("(inline content, marker={marker})"),
            })?;
        let block = &content[after_begin..after_begin + end_pos];
        // Trim leading newline that we add during upsert
        let block = block.strip_prefix('\n').unwrap_or(block);
        Ok(Some(block.to_string()))
    } else {
        Ok(None)
    }
}

/// Check if content contains a managed block with the given marker.
pub fn has_managed_block(content: &str, marker: &str) -> bool {
    let begin_tag = format!("{BEGIN_PREFIX}{marker}{MARKER_SUFFIX}");
    content.contains(&begin_tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_new_block() {
        let content = "# My File\n\nSome content.\n";
        let result = upsert_managed_block(content, "contracts", "- Contract A\n- Contract B\n").unwrap();
        assert!(result.contains("<!-- lexicon:begin:contracts -->"));
        assert!(result.contains("- Contract A\n- Contract B\n"));
        assert!(result.contains("<!-- lexicon:end:contracts -->"));
    }

    #[test]
    fn test_update_existing_block() {
        let content = "# My File\n\n<!-- lexicon:begin:contracts -->\nold content\n<!-- lexicon:end:contracts -->\n\nMore text.\n";
        let result = upsert_managed_block(content, "contracts", "new content\n").unwrap();
        assert!(result.contains("new content"));
        assert!(!result.contains("old content"));
        assert!(result.contains("More text."));
    }

    #[test]
    fn test_extract_block() {
        let content = "before\n<!-- lexicon:begin:test -->\nhello world\n<!-- lexicon:end:test -->\nafter\n";
        let block = extract_managed_block(content, "test").unwrap().unwrap();
        assert_eq!(block, "hello world\n");
    }

    #[test]
    fn test_extract_missing_block() {
        let content = "no blocks here\n";
        let block = extract_managed_block(content, "test").unwrap();
        assert!(block.is_none());
    }

    #[test]
    fn test_has_managed_block() {
        let content = "<!-- lexicon:begin:foo -->\nstuff\n<!-- lexicon:end:foo -->\n";
        assert!(has_managed_block(content, "foo"));
        assert!(!has_managed_block(content, "bar"));
    }

    #[test]
    fn test_malformed_block_error() {
        let content = "<!-- lexicon:begin:broken -->\nmissing end marker\n";
        let result = upsert_managed_block(content, "broken", "new");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_blocks() {
        let content = "";
        let result = upsert_managed_block(content, "first", "block 1\n").unwrap();
        let result = upsert_managed_block(&result, "second", "block 2\n").unwrap();
        assert!(has_managed_block(&result, "first"));
        assert!(has_managed_block(&result, "second"));
        assert_eq!(extract_managed_block(&result, "first").unwrap().unwrap(), "block 1\n");
        assert_eq!(extract_managed_block(&result, "second").unwrap().unwrap(), "block 2\n");
    }
}
