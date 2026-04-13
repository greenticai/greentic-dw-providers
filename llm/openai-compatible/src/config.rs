use greentic_dw_llm::LlmProviderFeatures;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::OpenAiCompatibleError;

/// Compatibility mode exposed by the target gateway.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAiCompatMode {
    /// Responses-style API.
    Responses,
    /// Chat Completions-style API.
    ChatCompletions,
}

impl OpenAiCompatMode {
    /// Returns the mode as a stable string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::ChatCompletions => "chat_completions",
        }
    }
}

/// Generic OpenAI-compatible provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiCompatibleConfig {
    /// Base URL for the target gateway.
    pub base_url: String,
    /// Optional bearer token or secret reference.
    pub api_key_secret: Option<String>,
    /// Default model identifier.
    pub model: String,
    /// Optional extra headers.
    pub headers: Map<String, Value>,
    /// Compatibility mode.
    pub compat_mode: OpenAiCompatMode,
    /// Whether stateful responses are supported.
    pub supports_stateful_responses: bool,
    /// Whether structured outputs are supported.
    pub supports_structured_outputs: bool,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether tool calling is allowed.
    pub allow_tools: bool,
    /// Whether streaming is allowed.
    pub allow_streaming: bool,
}

impl OpenAiCompatibleConfig {
    /// Creates a minimal valid configuration.
    #[must_use]
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            base_url: base_url.into(),
            api_key_secret: None,
            model: model.into(),
            headers: Map::new(),
            compat_mode: OpenAiCompatMode::Responses,
            supports_stateful_responses: false,
            supports_structured_outputs: false,
            timeout_ms,
            allow_tools: true,
            allow_streaming: true,
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), OpenAiCompatibleError> {
        if !(self.base_url.starts_with("http://") || self.base_url.starts_with("https://")) {
            return Err(OpenAiCompatibleError::config(
                "base_url must start with http:// or https://",
            ));
        }
        if self.model.trim().is_empty() {
            return Err(OpenAiCompatibleError::config("model must not be empty"));
        }
        if self.timeout_ms == 0 {
            return Err(OpenAiCompatibleError::config(
                "timeout_ms must be greater than zero",
            ));
        }
        Ok(())
    }

    /// Returns the effective feature profile.
    #[must_use]
    pub fn feature_profile(&self) -> LlmProviderFeatures {
        LlmProviderFeatures::new(
            true,
            self.supports_structured_outputs,
            self.allow_tools,
            self.allow_streaming,
            false,
            matches!(self.compat_mode, OpenAiCompatMode::Responses)
                && self.supports_stateful_responses,
            true,
            false,
        )
    }

    /// Returns the base URL without a trailing slash.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// Returns the selected endpoint URL.
    #[must_use]
    pub fn endpoint_url(&self) -> String {
        match self.compat_mode {
            OpenAiCompatMode::Responses => format!("{}/responses", self.base_url()),
            OpenAiCompatMode::ChatCompletions => format!("{}/chat/completions", self.base_url()),
        }
    }
}

/// Returns a JSON-schema-style fixture for the compatibility provider config.
#[must_use]
pub fn openai_compatible_config_schema() -> Value {
    json!({
        "type": "object",
        "required": ["base_url", "model", "compat_mode", "timeout_ms"],
        "properties": {
            "base_url": { "type": "string", "format": "uri" },
            "api_key_secret": { "type": "string" },
            "model": { "type": "string", "minLength": 1 },
            "headers": { "type": "object" },
            "compat_mode": {
                "type": "string",
                "enum": ["responses", "chat_completions"]
            },
            "supports_stateful_responses": { "type": "boolean" },
            "supports_structured_outputs": { "type": "boolean" },
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_streaming": { "type": "boolean" }
        },
        "additionalProperties": false
    })
}
