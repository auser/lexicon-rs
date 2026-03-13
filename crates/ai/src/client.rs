//! Claude API client using OAuth tokens from the auth system.

use crate::error::{AiError, AiResult};

/// A live AI client that calls the Claude API using stored OAuth credentials.
pub struct ClaudeClient {
    access_token: String,
    model: String,
}

impl ClaudeClient {
    /// Create a new client from an access token.
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }

    /// Override the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Send a prompt to Claude and return the text response.
    fn send(&self, system: &str, user_message: &str) -> AiResult<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| AiError::RequestFailed {
                reason: format!("building HTTP client: {e}"),
            })?;

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 8192,
            "system": system,
            "messages": [
                { "role": "user", "content": user_message }
            ]
        });

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| AiError::RequestFailed {
                reason: format!("sending request: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(AiError::RequestFailed {
                reason: format!("API returned {status}: {text}"),
            });
        }

        let json: serde_json::Value = resp.json().map_err(|e| AiError::RequestFailed {
            reason: format!("parsing response: {e}"),
        })?;

        // Extract text from the first content block
        let text = json["content"]
            .as_array()
            .and_then(|blocks| {
                blocks.iter().find_map(|b| {
                    if b["type"].as_str() == Some("text") {
                        b["text"].as_str().map(ToOwned::to_owned)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| AiError::RequestFailed {
                reason: "no text content in response".into(),
            })?;

        Ok(text)
    }
}

impl crate::boundary::AiProvider for ClaudeClient {
    fn complete(&self, system: &str, user_message: &str) -> AiResult<String> {
        self.send(system, user_message)
    }

    fn suggest_improvement(&self, context: &str, failure: &str) -> AiResult<String> {
        let system = "You are an expert software architect. Analyze the repository state \
            and verification failure, then suggest specific, actionable improvements. \
            Focus on additive changes that strengthen contracts, improve coverage, \
            or fix failing verification. Be concise.";

        let user_msg = format!(
            "## Repository State\n{context}\n\n## Verification Failure\n{failure}"
        );
        self.send(system, &user_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation() {
        let client = ClaudeClient::new("test-token".into());
        assert_eq!(client.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn client_with_model() {
        let client = ClaudeClient::new("tok".into()).with_model("claude-opus-4-20250514");
        assert_eq!(client.model, "claude-opus-4-20250514");
    }
}
