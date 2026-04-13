use greentic_dw_llm::LlmProviderFeatures;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::BedrockError;

/// AWS auth mode used to resolve Bedrock credentials.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum BedrockAuthMode {
    /// Use the AWS default credential chain.
    DefaultChain,
    /// Use a named local AWS profile.
    Profile {
        /// Shared config/credentials profile name.
        profile_name: String,
    },
    /// Use static access-key style credentials.
    StaticKeys {
        /// Access key id or secret reference.
        access_key_id_secret: String,
        /// Secret access key or secret reference.
        secret_access_key_secret: String,
        /// Optional session token or secret reference.
        session_token_secret: Option<String>,
    },
}

/// Bedrock provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BedrockConfig {
    /// AWS region.
    pub region: String,
    /// Default Bedrock model id or inference profile id.
    pub model_id: String,
    /// Credential resolution mode.
    pub auth_mode: BedrockAuthMode,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether tool calling is allowed.
    pub allow_tools: bool,
    /// Whether streaming requests are allowed.
    pub allow_streaming: bool,
}

impl BedrockConfig {
    /// Creates a minimal valid configuration using the default AWS credential chain.
    #[must_use]
    pub fn new(region: impl Into<String>, model_id: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            region: region.into(),
            model_id: model_id.into(),
            auth_mode: BedrockAuthMode::DefaultChain,
            timeout_ms,
            allow_tools: true,
            allow_streaming: true,
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), BedrockError> {
        if self.region.trim().is_empty() {
            return Err(BedrockError::config("region must not be empty"));
        }
        if self.model_id.trim().is_empty() {
            return Err(BedrockError::config("model_id must not be empty"));
        }
        if self.timeout_ms == 0 {
            return Err(BedrockError::config("timeout_ms must be greater than zero"));
        }
        match &self.auth_mode {
            BedrockAuthMode::DefaultChain => {}
            BedrockAuthMode::Profile { profile_name } => {
                if profile_name.trim().is_empty() {
                    return Err(BedrockError::config("profile_name must not be empty"));
                }
            }
            BedrockAuthMode::StaticKeys {
                access_key_id_secret,
                secret_access_key_secret,
                ..
            } => {
                if access_key_id_secret.trim().is_empty()
                    || secret_access_key_secret.trim().is_empty()
                {
                    return Err(BedrockError::config(
                        "static access key fields must not be empty",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the feature profile implied by this configuration.
    #[must_use]
    pub fn feature_profile(&self) -> LlmProviderFeatures {
        LlmProviderFeatures::new(
            true,
            false,
            self.allow_tools,
            self.allow_streaming,
            false,
            false,
            false,
            true,
        )
    }
}

/// Returns a JSON-schema-style fixture for the Bedrock provider config.
#[must_use]
pub fn bedrock_config_schema() -> Value {
    json!({
        "type": "object",
        "required": ["region", "model_id", "auth_mode", "timeout_ms", "allow_tools", "allow_streaming"],
        "properties": {
            "region": { "type": "string", "minLength": 1 },
            "model_id": { "type": "string", "minLength": 1 },
            "auth_mode": {
                "oneOf": [
                    { "type": "object", "properties": { "mode": { "const": "default_chain" } }, "required": ["mode"] },
                    {
                        "type": "object",
                        "properties": {
                            "mode": { "const": "profile" },
                            "profile_name": { "type": "string", "minLength": 1 }
                        },
                        "required": ["mode", "profile_name"]
                    },
                    {
                        "type": "object",
                        "properties": {
                            "mode": { "const": "static_keys" },
                            "access_key_id_secret": { "type": "string", "minLength": 1 },
                            "secret_access_key_secret": { "type": "string", "minLength": 1 },
                            "session_token_secret": { "type": "string" }
                        },
                        "required": ["mode", "access_key_id_secret", "secret_access_key_secret"]
                    }
                ]
            },
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_streaming": { "type": "boolean" }
        },
        "additionalProperties": false
    })
}
