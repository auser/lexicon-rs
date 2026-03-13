use crate::error::AiResult;

/// Trait for AI provider integration.
///
/// This defines the boundary between lexicon and external AI services.
/// In v1, no live AI provider is implemented — prompts are prepared
/// for manual use with Claude Code or future API integration.
pub trait AiProvider {
    /// Enhance a proposed artifact draft with AI suggestions.
    fn enhance_proposal(&self, prompt: &str, context: &str) -> AiResult<String>;

    /// Generate an improvement suggestion for a failing verification.
    fn suggest_improvement(&self, context: &str, failure: &str) -> AiResult<String>;
}

/// A no-op AI provider that always returns the input unchanged.
///
/// This is the default when no AI is configured, ensuring all
/// workflows work fully without AI.
pub struct NoOpProvider;

impl AiProvider for NoOpProvider {
    fn enhance_proposal(&self, _prompt: &str, _context: &str) -> AiResult<String> {
        Err(crate::error::AiError::NotAvailable)
    }

    fn suggest_improvement(&self, _context: &str, _failure: &str) -> AiResult<String> {
        Err(crate::error::AiError::NotAvailable)
    }
}
