use greentic_dw_llm::LlmProviderFeatures;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AnthropicError;

/// Default Anthropic API base URL.
pub const ANTHROPIC_DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";

/// Anthropic provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// API key secret or secret reference.
    pub api_key_secret: String,
    /// Optional base URL override for proxying.
    pub base_url: Option<String>,
    /// Default model identifier.
    pub model: String,
    /// Default max tokens value required by the Messages API.
    pub max_tokens: u32,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether tool use is allowed.
    pub allow_tools: bool,
    /// Whether structured outputs are allowed through synthetic tool forcing.
    pub allow_structured_outputs: bool,
    /// Whether extended thinking is allowed.
    pub allow_thinking: bool,
    /// Optional budget tokens for Anthropic thinking mode.
    pub thinking_budget_tokens: Option<u32>,
}

impl AnthropicConfig {
    /// Creates a new configuration with required fields.
    #[must_use]
    pub fn new(
        api_key_secret: impl Into<String>,
        model: impl Into<String>,
        max_tokens: u32,
        timeout_ms: u64,
    ) -> Self {
        Self {
            api_key_secret: api_key_secret.into(),
            base_url: None,
            model: model.into(),
            max_tokens,
            timeout_ms,
            allow_tools: true,
            allow_structured_outputs: true,
            allow_thinking: false,
            thinking_budget_tokens: None,
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), AnthropicError> {
        if self.api_key_secret.trim().is_empty() {
            return Err(AnthropicError::config("api_key_secret must not be empty"));
        }
        if self.model.trim().is_empty() {
            return Err(AnthropicError::config("model must not be empty"));
        }
        if self.max_tokens == 0 {
            return Err(AnthropicError::config(
                "max_tokens must be greater than zero",
            ));
        }
        if self.timeout_ms == 0 {
            return Err(AnthropicError::config(
                "timeout_ms must be greater than zero",
            ));
        }
        if let Some(base_url) = &self.base_url {
            validate_https_override(base_url)?;
        }
        if self.allow_thinking && self.thinking_budget_tokens.unwrap_or(0) == 0 {
            return Err(AnthropicError::config(
                "thinking_budget_tokens must be set when allow_thinking is enabled",
            ));
        }
        Ok(())
    }

    /// Returns the effective base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or(ANTHROPIC_DEFAULT_BASE_URL)
            .trim_end_matches('/')
    }

    /// Returns the Messages API endpoint URL.
    #[must_use]
    pub fn messages_url(&self) -> String {
        format!("{}/messages", self.base_url())
    }

    /// Returns the feature profile implied by this configuration.
    #[must_use]
    pub fn feature_profile(&self) -> LlmProviderFeatures {
        LlmProviderFeatures::new(
            true,
            self.allow_structured_outputs,
            self.allow_tools,
            true,
            false,
            false,
            false,
            true,
        )
    }
}

/// Returns a JSON-schema-style fixture for the Anthropic provider config.
#[must_use]
pub fn anthropic_config_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "api_key_secret",
            "model",
            "max_tokens",
            "timeout_ms",
            "allow_tools",
            "allow_structured_outputs",
            "allow_thinking"
        ],
        "properties": {
            "api_key_secret": { "type": "string", "minLength": 1 },
            "base_url": { "type": "string", "format": "uri" },
            "model": { "type": "string", "minLength": 1 },
            "max_tokens": { "type": "integer", "minimum": 1 },
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_structured_outputs": { "type": "boolean" },
            "allow_thinking": { "type": "boolean" },
            "thinking_budget_tokens": { "type": "integer", "minimum": 1 }
        },
        "additionalProperties": false
    })
}

fn validate_https_override(base_url: &str) -> Result<(), AnthropicError> {
    let parsed = Url::parse(base_url)
        .map_err(|_| AnthropicError::config("base_url must be a valid absolute URL"))?;
    if parsed.scheme() != "https" {
        return Err(AnthropicError::config(
            "base_url must use https for the Anthropic provider",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::AnthropicConfig;

    #[test]
    fn validate_rejects_http_override() {
        let mut config = AnthropicConfig::new("secret", "claude-sonnet", 256, 5_000);
        config.base_url = Some("http://example.com/v1".to_string());
        let err = config.validate().expect_err("http override should fail");
        assert_eq!(
            err.to_string(),
            "base_url must use https for the Anthropic provider"
        );
    }
}
