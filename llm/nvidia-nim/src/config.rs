use greentic_dw_llm::LlmProviderFeatures;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::NvidiaNimError;

/// Inference and management mode for the target deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NvidiaNimApiMode {
    /// Use only the OpenAI-compatible inference surface.
    OpenAiCompatible,
    /// Use inference plus NIM-specific discovery and health helpers.
    NimAware,
}

/// Generic NIM feature overrides used when runtime detection is unavailable.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvidiaNimFeatureOverride {
    /// Override tool calling support.
    pub tool_calling: Option<bool>,
    /// Override structured output support.
    pub structured_outputs: Option<bool>,
    /// Override streaming support.
    pub streaming: Option<bool>,
}

/// NVIDIA NIM provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvidiaNimConfig {
    /// Base URL for the NIM endpoint.
    pub base_url: String,
    /// Inference and management mode.
    pub api_mode: NvidiaNimApiMode,
    /// Default model identifier.
    pub model: String,
    /// Optional bearer token or secret reference.
    pub api_key_secret: Option<String>,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether tool calling is allowed.
    pub allow_tools: bool,
    /// Whether structured outputs are allowed.
    pub allow_structured_outputs: bool,
    /// Whether streaming is allowed.
    pub allow_streaming: bool,
    /// Whether to try model discovery on startup or through helper methods.
    pub discover_models_on_start: bool,
    /// Whether to try health checks on startup or through helper methods.
    pub healthcheck_on_start: bool,
    /// Optional extra headers.
    pub headers: Map<String, Value>,
    /// Optional manual feature overrides.
    pub features_override: Option<NvidiaNimFeatureOverride>,
}

impl NvidiaNimConfig {
    /// Creates a minimal valid configuration.
    #[must_use]
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            base_url: base_url.into(),
            api_mode: NvidiaNimApiMode::OpenAiCompatible,
            model: model.into(),
            api_key_secret: None,
            timeout_ms,
            allow_tools: true,
            allow_structured_outputs: false,
            allow_streaming: true,
            discover_models_on_start: false,
            healthcheck_on_start: false,
            headers: Map::new(),
            features_override: None,
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), NvidiaNimError> {
        if !(self.base_url.starts_with("http://") || self.base_url.starts_with("https://")) {
            return Err(NvidiaNimError::config(
                "base_url must start with http:// or https://",
            ));
        }
        if self.model.trim().is_empty() {
            return Err(NvidiaNimError::config("model must not be empty"));
        }
        if self.timeout_ms == 0 {
            return Err(NvidiaNimError::config(
                "timeout_ms must be greater than zero",
            ));
        }
        if matches!(self.api_mode, NvidiaNimApiMode::OpenAiCompatible)
            && (self.discover_models_on_start || self.healthcheck_on_start)
        {
            return Err(NvidiaNimError::config(
                "discover_models_on_start and healthcheck_on_start require api_mode=nim_aware",
            ));
        }
        Ok(())
    }

    /// Returns whether the provider is configured for NIM-aware helpers.
    #[must_use]
    pub const fn is_nim_aware(&self) -> bool {
        matches!(self.api_mode, NvidiaNimApiMode::NimAware)
    }

    /// Returns the base URL without a trailing slash.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// Returns the OpenAI-compatible chat completions endpoint.
    #[must_use]
    pub fn inference_url(&self) -> String {
        format!("{}/chat/completions", self.base_url())
    }

    /// Returns the model discovery endpoint.
    #[must_use]
    pub fn models_url(&self) -> String {
        format!("{}/models", self.base_url())
    }

    /// Returns the ready health endpoint.
    #[must_use]
    pub fn ready_url(&self) -> String {
        format!("{}/health/ready", self.base_url())
    }

    /// Returns the live health endpoint.
    #[must_use]
    pub fn live_url(&self) -> String {
        format!("{}/health/live", self.base_url())
    }

    /// Returns the feature profile implied by this configuration.
    #[must_use]
    pub fn feature_profile(&self) -> LlmProviderFeatures {
        let overrides = self.features_override.clone().unwrap_or_default();
        LlmProviderFeatures::new(
            true,
            overrides
                .structured_outputs
                .unwrap_or(self.allow_structured_outputs),
            overrides.tool_calling.unwrap_or(self.allow_tools),
            overrides.streaming.unwrap_or(self.allow_streaming),
            false,
            false,
            true,
            false,
        )
    }
}

/// Returns a JSON-schema-style fixture for the NIM provider config.
#[must_use]
pub fn nvidia_nim_config_schema() -> Value {
    json!({
        "type": "object",
        "required": ["base_url", "api_mode", "model", "timeout_ms"],
        "properties": {
            "base_url": { "type": "string", "format": "uri" },
            "api_mode": { "type": "string", "enum": ["openai_compatible", "nim_aware"] },
            "model": { "type": "string", "minLength": 1 },
            "api_key_secret": { "type": "string" },
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_structured_outputs": { "type": "boolean" },
            "allow_streaming": { "type": "boolean" },
            "discover_models_on_start": { "type": "boolean" },
            "healthcheck_on_start": { "type": "boolean" },
            "headers": { "type": "object" },
            "features_override": { "type": "object" }
        },
        "additionalProperties": false
    })
}
