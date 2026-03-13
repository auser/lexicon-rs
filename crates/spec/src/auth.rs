use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Supported AI authentication providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Claude,
    OpenAi,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::OpenAi => "openai",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Claude => "Anthropic / Claude",
            Self::OpenAi => "OpenAI",
        }
    }

    pub fn config(self) -> ProviderConfig {
        match self {
            Self::Claude => ProviderConfig {
                client_id: "9d1c250a-e61b-44d9-88ed-5944d1962f5e",
                auth_url: "https://claude.ai/oauth/authorize",
                token_url: "https://platform.claude.com/v1/oauth/token",
                default_port: 54321,
                scopes: "user:inference user:profile",
                token_exchange_json: true,
            },
            Self::OpenAi => ProviderConfig {
                client_id: "app_EMoamEEZ73f0CkXaXp7hrann",
                auth_url: "https://auth.openai.com/oauth/authorize",
                token_url: "https://auth.openai.com/oauth/token",
                default_port: 1455,
                scopes: "openid profile email offline_access",
                token_exchange_json: false,
            },
        }
    }

    pub const ALL: [Provider; 2] = [Provider::Claude, Provider::OpenAi];
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Ok(Self::Claude),
            "openai" => Ok(Self::OpenAi),
            other => Err(format!("unknown provider '{other}' — use 'claude' or 'openai'")),
        }
    }
}

/// OAuth configuration for a provider.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub client_id: &'static str,
    pub auth_url: &'static str,
    pub token_url: &'static str,
    pub default_port: u16,
    pub scopes: &'static str,
    /// If true, token exchange uses JSON body (Claude). Otherwise form-encoded (OpenAI).
    pub token_exchange_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub provider: Provider,
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

impl Credentials {
    /// Returns true if the token has expired or will expire within 60 seconds.
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            exp <= now + 60
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_round_trip() {
        for p in Provider::ALL {
            let s = p.as_str();
            let parsed: Provider = s.parse().unwrap();
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn provider_display() {
        assert_eq!(Provider::Claude.to_string(), "claude");
        assert_eq!(Provider::OpenAi.to_string(), "openai");
    }

    #[test]
    fn credentials_not_expired_when_no_expiry() {
        let creds = Credentials {
            provider: Provider::Claude,
            access_token: "test".into(),
            refresh_token: None,
            expires_at: None,
        };
        assert!(!creds.is_expired());
    }

    #[test]
    fn credentials_expired_in_past() {
        let creds = Credentials {
            provider: Provider::Claude,
            access_token: "test".into(),
            refresh_token: None,
            expires_at: Some(1000),
        };
        assert!(creds.is_expired());
    }

    #[test]
    fn credentials_not_expired_far_future() {
        let creds = Credentials {
            provider: Provider::Claude,
            access_token: "test".into(),
            refresh_token: None,
            expires_at: Some(u64::MAX / 2),
        };
        assert!(!creds.is_expired());
    }

    #[test]
    fn provider_config_has_valid_urls() {
        for p in Provider::ALL {
            let cfg = p.config();
            assert!(cfg.auth_url.starts_with("https://"));
            assert!(cfg.token_url.starts_with("https://"));
            assert!(!cfg.client_id.is_empty());
            assert!(cfg.default_port > 0);
        }
    }
}
