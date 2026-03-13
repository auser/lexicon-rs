use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::{SessionStatus, StepType, WorkflowKind};
use crate::version::SchemaVersion;

/// A conversation session record.
///
/// Records the steps, decisions, and context of a conversational
/// artifact creation workflow. Stored for reuse in future generations.
///
/// Stored at `.lexicon/conversations/<uuid>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub schema_version: SchemaVersion,
    pub id: Uuid,
    pub workflow: WorkflowKind,
    pub status: SessionStatus,
    #[serde(default)]
    pub steps: Vec<SessionStep>,
    #[serde(default)]
    pub decisions: Vec<Decision>,
    /// The artifact ID produced by this session, if any.
    #[serde(default)]
    pub artifact_id: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
}

/// A single step in a conversation workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStep {
    pub step_type: StepType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// A decision recorded during a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub rationale: Option<String>,
}

impl ConversationSession {
    /// Create a new active session for the given workflow.
    pub fn new(workflow: WorkflowKind) -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            id: Uuid::new_v4(),
            workflow,
            status: SessionStatus::Active,
            steps: Vec::new(),
            decisions: Vec::new(),
            artifact_id: None,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Add a step to the session.
    pub fn add_step(&mut self, step_type: StepType, content: String) {
        self.steps.push(SessionStep {
            step_type,
            content,
            timestamp: Utc::now(),
        });
    }

    /// Record a decision.
    pub fn add_decision(&mut self, key: String, value: String, rationale: Option<String>) {
        self.decisions.push(Decision {
            key,
            value,
            rationale,
        });
    }

    /// Mark the session as completed.
    pub fn complete(&mut self, artifact_id: Option<String>) {
        self.status = SessionStatus::Completed;
        self.artifact_id = artifact_id;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the session as abandoned.
    pub fn abandon(&mut self) {
        self.status = SessionStatus::Abandoned;
        self.completed_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let mut session = ConversationSession::new(WorkflowKind::ContractNew);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.completed_at.is_none());

        session.add_step(StepType::Question, "What is the contract title?".to_string());
        session.add_step(StepType::UserInput, "Key-Value Store".to_string());
        session.add_decision(
            "title".to_string(),
            "Key-Value Store".to_string(),
            Some("User provided".to_string()),
        );

        assert_eq!(session.steps.len(), 2);
        assert_eq!(session.decisions.len(), 1);

        session.complete(Some("key-value-store".to_string()));
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.completed_at.is_some());
        assert_eq!(session.artifact_id.as_deref(), Some("key-value-store"));
    }

    #[test]
    fn test_session_json_roundtrip() {
        let mut session = ConversationSession::new(WorkflowKind::Init);
        session.add_step(StepType::Info, "Initializing repo".to_string());
        session.complete(None);

        let json = serde_json::to_string_pretty(&session).unwrap();
        let parsed: ConversationSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, session.id);
        assert_eq!(parsed.steps.len(), 1);
    }
}
