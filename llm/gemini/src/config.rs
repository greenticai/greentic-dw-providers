use greentic_dw_llm::LlmProviderFeatures;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::GeminiError;

/// Default Gemini REST base URL.
pub const GEMINI_DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Gemini provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// API key secret or secret reference.
    pub api_key_secret: String,
    /// Optional base URL override for proxies or gateways.
    pub base_url: Option<String>,
    /// Default model identifier.
    pub model: String,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether tool calling is allowed.
    pub allow_tools: bool,
    /// Whether structured outputs are allowed.
    pub allow_structured_outputs: bool,
    /// Optional provider-owned safety profile selection.
    pub safety_profile: Option<String>,
}

impl GeminiConfig {
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
            timeout_ms,
            allow_tools: true,
            allow_structured_outputs: true,
            safety_profile: None,
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), GeminiError> {
        if self.api_key_secret.trim().is_empty() {
            return Err(GeminiError::config("api_key_secret must not be empty"));
        }
        if self.model.trim().is_empty() {
            return Err(GeminiError::config("model must not be empty"));
        }
        if self.timeout_ms == 0 {
            return Err(GeminiError::config("timeout_ms must be greater than zero"));
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
            .unwrap_or(GEMINI_DEFAULT_BASE_URL)
            .trim_end_matches('/')
    }

    /// Returns the `generateContent` endpoint URL for the configured model.
    #[must_use]
    pub fn generate_content_url(&self, model: &str) -> String {
        format!(
            "{}/{}:generateContent?key={}",
            self.base_url(),
            model,
            self.api_key_secret
        )
    }

    /// Returns the feature profile implied by this configuration.
    #[must_use]
    pub fn feature_profile(&self) -> LlmProviderFeatures {
        LlmProviderFeatures::new(
            true,
            self.allow_structured_outputs,
            self.allow_tools,
            false,
            false,
            false,
            false,
            false,
        )
    }
}

/// Returns a JSON-schema-style fixture for the Gemini provider config.
#[must_use]
pub fn gemini_config_schema() -> Value {
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
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_structured_outputs": { "type": "boolean" },
            "safety_profile": { "type": "string" }
        },
        "additionalProperties": false
    })
}

fn validate_https_override(base_url: &str) -> Result<(), GeminiError> {
    let parsed = Url::parse(base_url)
        .map_err(|_| GeminiError::config("base_url must be a valid absolute URL"))?;
    if parsed.scheme() != "https" {
        return Err(GeminiError::config(
            "base_url must use https for the Gemini provider",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::GeminiConfig;

    #[test]
    fn validate_rejects_http_override() {
        let mut config = GeminiConfig::new("secret", "gemini-2.5-pro", 5_000);
        config.base_url = Some("http://example.com/v1beta/models".to_string());
        let err = config.validate().expect_err("http override should fail");
        assert_eq!(
            err.to_string(),
            "base_url must use https for the Gemini provider"
        );
    }
}
