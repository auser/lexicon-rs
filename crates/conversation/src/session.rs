use std::path::Path;

use lexicon_spec::session::ConversationSession;

use crate::error::ConversationResult;

/// Save a conversation session to the conversations directory.
pub fn save_session(conversations_dir: &Path, session: &ConversationSession) -> ConversationResult<()> {
    let filename = format!("{}.json", session.id);
    let path = conversations_dir.join(filename);
    let content = serde_json::to_string_pretty(session)?;
    lexicon_fs::safe_write::safe_write(&path, &content, false)?;
    Ok(())
}

/// Load a conversation session by ID.
pub fn load_session(
    conversations_dir: &Path,
    id: &uuid::Uuid,
) -> ConversationResult<Option<ConversationSession>> {
    let filename = format!("{id}.json");
    let path = conversations_dir.join(filename);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let session: ConversationSession = serde_json::from_str(&content)?;
    Ok(Some(session))
}

/// List all session files in the conversations directory.
pub fn list_sessions(conversations_dir: &Path) -> ConversationResult<Vec<ConversationSession>> {
    if !conversations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(conversations_dir)? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|ext| ext == "json")
        {
            let content = std::fs::read_to_string(entry.path())?;
            if let Ok(session) = serde_json::from_str::<ConversationSession>(&content) {
                sessions.push(session);
            }
        }
    }

    sessions.sort_by_key(|s| s.started_at);
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::common::WorkflowKind;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_session() {
        let dir = TempDir::new().unwrap();
        let mut session = ConversationSession::new(WorkflowKind::Init);
        session.complete(None);

        save_session(dir.path(), &session).unwrap();
        let loaded = load_session(dir.path(), &session.id).unwrap().unwrap();
        assert_eq!(loaded.id, session.id);
    }

    #[test]
    fn test_load_missing_session() {
        let dir = TempDir::new().unwrap();
        let id = uuid::Uuid::new_v4();
        let result = load_session(dir.path(), &id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_sessions() {
        let dir = TempDir::new().unwrap();

        let s1 = ConversationSession::new(WorkflowKind::Init);
        let s2 = ConversationSession::new(WorkflowKind::ContractNew);
        save_session(dir.path(), &s1).unwrap();
        save_session(dir.path(), &s2).unwrap();

        let sessions = list_sessions(dir.path()).unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
