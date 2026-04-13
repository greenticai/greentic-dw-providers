use greentic_dw_llm::LlmProviderFeatures;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::OpenAiError;

/// Default OpenAI API base URL.
pub const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// Optional reasoning effort profile for supported models.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAiReasoningProfile {
    /// Lower-latency reasoning.
    Low,
    /// Balanced reasoning.
    Medium,
    /// Higher-effort reasoning.
    High,
}

impl OpenAiReasoningProfile {
    /// Returns the profile as the API wire value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Native OpenAI provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiConfig {
    /// API key secret or secret reference.
    pub api_key_secret: String,
    /// Optional override for the API base URL.
    pub base_url: Option<String>,
    /// Default model identifier.
    pub model: String,
    /// Optional organization header.
    pub organization: Option<String>,
    /// Optional project header.
    pub project: Option<String>,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether function tools are allowed.
    pub allow_tools: bool,
    /// Whether structured outputs are allowed.
    pub allow_structured_outputs: bool,
    /// Optional reasoning profile.
    pub reasoning_profile: Option<OpenAiReasoningProfile>,
}

impl OpenAiConfig {
    /// Creates a new configuration with required fields.
    #[must_use]
    pub fn new(
        api_key_secret: impl Into<String>,
        model: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            api_key_secret: api_key_secret.into(),
            base_url: None,
            model: model.into(),
            organization: None,
            project: None,
            timeout_ms,
            allow_tools: true,
            allow_structured_outputs: true,
            reasoning_profile: None,
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), OpenAiError> {
        if self.api_key_secret.trim().is_empty() {
            return Err(OpenAiError::config("api_key_secret must not be empty"));
        }
        if self.model.trim().is_empty() {
            return Err(OpenAiError::config("model must not be empty"));
        }
        if self.timeout_ms == 0 {
            return Err(OpenAiError::config("timeout_ms must be greater than zero"));
        }
        if let Some(base_url) = &self.base_url {
            validate_https_override(base_url)?;
        }
        Ok(())
    }

    /// Returns the effective base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or(OPENAI_DEFAULT_BASE_URL)
            .trim_end_matches('/')
    }

    /// Returns the Responses API endpoint URL.
    #[must_use]
    pub fn responses_url(&self) -> String {
        format!("{}/responses", self.base_url())
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
            true,
            false,
            false,
        )
    }
}

/// Returns a JSON-schema-style fixture for the native OpenAI provider config.
#[must_use]
pub fn openai_config_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "api_key_secret",
            "model",
            "timeout_ms",
            "allow_tools",
            "allow_structured_outputs"
        ],
        "properties": {
            "api_key_secret": { "type": "string", "minLength": 1 },
            "base_url": { "type": "string", "format": "uri" },
            "model": { "type": "string", "minLength": 1 },
            "organization": { "type": "string" },
            "project": { "type": "string" },
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_structured_outputs": { "type": "boolean" },
            "reasoning_profile": {
                "type": "string",
                "enum": ["low", "medium", "high"]
            }
        },
        "additionalProperties": false
    })
}

fn validate_https_override(base_url: &str) -> Result<(), OpenAiError> {
    let parsed = Url::parse(base_url)
        .map_err(|_| OpenAiError::config("base_url must be a valid absolute URL"))?;
    if parsed.scheme() != "https" {
        return Err(OpenAiError::config(
            "base_url must use https for the native OpenAI provider",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::OpenAiConfig;

    #[test]
    fn validate_rejects_http_override() {
        let mut config = OpenAiConfig::new("secret", "gpt-5.4", 5_000);
        config.base_url = Some("http://example.com/v1".to_string());
        let err = config.validate().expect_err("http override should fail");
        assert_eq!(
            err.to_string(),
            "base_url must use https for the native OpenAI provider"
        );
    }
}
