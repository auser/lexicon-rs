use lexicon_repo::layout::RepoLayout;

use crate::error::ScaffoldResult;

/// Generate the content for the lexicon-managed CLAUDE.md blocks.
pub fn generate_claude_content(context: &str) -> String {
    let mut content = String::new();
    content.push_str("## Lexicon Verification Context\n\n");
    content.push_str("This section is managed by `lexicon sync claude`. Do not edit manually.\n\n");
    content.push_str(context);
    content
}

/// Write or update the CLAUDE.md file with managed blocks.
pub fn sync_claude_md(layout: &RepoLayout, context: &str) -> ScaffoldResult<()> {
    let path = layout.claude_md_path();
    let managed_content = generate_claude_content(context);

    let existing = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let updated = lexicon_fs::patch::upsert_managed_block(
        &existing,
        "lexicon-context",
        &managed_content,
    )?;

    lexicon_fs::safe_write::safe_write(&path, &updated, false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_claude_md_new_file() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        sync_claude_md(&layout, "test context\n").unwrap();

        let content = std::fs::read_to_string(layout.claude_md_path()).unwrap();
        assert!(content.contains("lexicon:begin:lexicon-context"));
        assert!(content.contains("test context"));
        assert!(content.contains("lexicon:end:lexicon-context"));
    }

    #[test]
    fn test_sync_claude_md_preserves_existing() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        // Write existing content
        std::fs::write(layout.claude_md_path(), "# My Project\n\nCustom content.\n").unwrap();

        sync_claude_md(&layout, "lexicon info\n").unwrap();

        let content = std::fs::read_to_string(layout.claude_md_path()).unwrap();
        assert!(content.contains("# My Project"));
        assert!(content.contains("Custom content."));
        assert!(content.contains("lexicon info"));
    }

    #[test]
    fn test_sync_claude_md_updates_existing_block() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        sync_claude_md(&layout, "first version\n").unwrap();
        sync_claude_md(&layout, "updated version\n").unwrap();

        let content = std::fs::read_to_string(layout.claude_md_path()).unwrap();
        assert!(content.contains("updated version"));
        assert!(!content.contains("first version"));
    }
}
