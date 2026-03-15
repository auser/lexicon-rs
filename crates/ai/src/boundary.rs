use crate::error::AiResult;

/// Trait for AI provider integration.
///
/// This defines the boundary between lexicon and external AI services.
pub trait AiProvider {
    /// Generate an artifact or enhancement given a system prompt and user message.
    fn complete(&self, system: &str, user_message: &str) -> AiResult<String>;

    /// Generate an improvement suggestion for a failing verification.
    fn suggest_improvement(&self, context: &str, failure: &str) -> AiResult<String>;

    /// Return the model identifier currently in use.
    fn model_id(&self) -> &str {
        "unknown"
    }
}

/// A no-op AI provider that always returns the input unchanged.
///
/// This is the default when no AI is configured, ensuring all
/// workflows work fully without AI.
pub struct NoOpProvider;

impl AiProvider for NoOpProvider {
    fn complete(&self, _system: &str, _user_message: &str) -> AiResult<String> {
        Err(crate::error::AiError::NotAvailable)
    }

    fn suggest_improvement(&self, _context: &str, _failure: &str) -> AiResult<String> {
        Err(crate::error::AiError::NotAvailable)
    }
}
