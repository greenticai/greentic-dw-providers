use greentic_dw_llm::LlmProviderFeatures;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::AzureOpenAiError;

/// Default Azure OpenAI API version.
pub const AZURE_OPENAI_DEFAULT_API_VERSION: &str = "v1";

/// Default Microsoft Entra scope for Azure OpenAI.
pub const AZURE_OPENAI_DEFAULT_ENTRA_SCOPE: &str = "https://cognitiveservices.azure.com/.default";

/// Azure-specific authentication mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AzureOpenAiAuthMode {
    /// Authenticate with an Azure OpenAI API key.
    ApiKey,
    /// Authenticate with a Microsoft Entra bearer token.
    EntraId,
}

impl AzureOpenAiAuthMode {
    /// Returns the auth mode as a stable string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::EntraId => "entra_id",
        }
    }
}

/// Azure OpenAI provider configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AzureOpenAiConfig {
    /// Azure OpenAI resource endpoint.
    pub endpoint: String,
    /// Azure deployment name used as the first-class model identifier.
    pub deployment: String,
    /// Azure API version path segment.
    pub api_version: String,
    /// Selected authentication mode.
    pub auth_mode: AzureOpenAiAuthMode,
    /// Optional API key secret or secret reference.
    pub api_key_secret: Option<String>,
    /// Optional Entra bearer token secret or secret reference.
    pub entra_token_secret: Option<String>,
    /// Optional Entra scope for token acquisition workflows.
    pub entra_scope: Option<String>,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether tool calling is allowed.
    pub allow_tools: bool,
    /// Whether structured outputs are allowed.
    pub allow_structured_outputs: bool,
    /// Whether stateful responses are allowed when using the Responses API.
    pub allow_stateful_responses: bool,
    /// Whether to use Azure's Responses API.
    pub use_responses_api: bool,
    /// Optional extra headers.
    pub raw_headers: Map<String, Value>,
}

impl AzureOpenAiConfig {
    /// Creates a minimal valid Azure OpenAI configuration.
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        deployment: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            deployment: deployment.into(),
            api_version: AZURE_OPENAI_DEFAULT_API_VERSION.to_string(),
            auth_mode: AzureOpenAiAuthMode::ApiKey,
            api_key_secret: None,
            entra_token_secret: None,
            entra_scope: Some(AZURE_OPENAI_DEFAULT_ENTRA_SCOPE.to_string()),
            timeout_ms,
            allow_tools: true,
            allow_structured_outputs: true,
            allow_stateful_responses: true,
            use_responses_api: true,
            raw_headers: Map::new(),
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), AzureOpenAiError> {
        validate_https_endpoint(&self.endpoint)?;
        if self.deployment.trim().is_empty() {
            return Err(AzureOpenAiError::config("deployment must not be empty"));
        }
        if self.api_version.trim().is_empty() {
            return Err(AzureOpenAiError::config("api_version must not be empty"));
        }
        if self.timeout_ms == 0 {
            return Err(AzureOpenAiError::config(
                "timeout_ms must be greater than zero",
            ));
        }
        if self.allow_stateful_responses && !self.use_responses_api {
            return Err(AzureOpenAiError::config(
                "allow_stateful_responses requires use_responses_api = true",
            ));
        }

        match self.auth_mode {
            AzureOpenAiAuthMode::ApiKey => {
                if self
                    .api_key_secret
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    return Err(AzureOpenAiError::config(
                        "api_key_secret is required for api_key auth",
                    ));
                }
            }
            AzureOpenAiAuthMode::EntraId => {
                if self
                    .entra_token_secret
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    return Err(AzureOpenAiError::config(
                        "entra_token_secret is required for entra_id auth",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Returns the normalized feature profile implied by this config.
    #[must_use]
    pub fn feature_profile(&self) -> LlmProviderFeatures {
        LlmProviderFeatures::new(
            true,
            self.allow_structured_outputs,
            self.allow_tools,
            true,
            false,
            self.use_responses_api && self.allow_stateful_responses,
            false,
            true,
        )
    }

    /// Returns the normalized endpoint without a trailing slash.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.endpoint.trim_end_matches('/')
    }

    /// Returns the effective Entra scope.
    #[must_use]
    pub fn entra_scope(&self) -> &str {
        self.entra_scope
            .as_deref()
            .unwrap_or(AZURE_OPENAI_DEFAULT_ENTRA_SCOPE)
    }

    /// Returns the Responses API endpoint URL.
    #[must_use]
    pub fn responses_url(&self) -> String {
        format!(
            "{}/openai/{}/responses",
            self.endpoint(),
            self.api_version.trim_matches('/'),
        )
    }

    /// Returns the Chat Completions endpoint URL.
    #[must_use]
    pub fn chat_completions_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint(),
            self.deployment,
            self.api_version,
        )
    }
}

/// Returns a JSON-schema-style fixture for the Azure OpenAI provider config.
#[must_use]
pub fn azure_openai_config_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "endpoint",
            "deployment",
            "api_version",
            "auth_mode",
            "timeout_ms",
            "allow_tools",
            "allow_structured_outputs",
            "allow_stateful_responses",
            "use_responses_api"
        ],
        "properties": {
            "endpoint": { "type": "string", "format": "uri" },
            "deployment": { "type": "string", "minLength": 1 },
            "api_version": { "type": "string", "minLength": 1 },
            "auth_mode": { "type": "string", "enum": ["api_key", "entra_id"] },
            "api_key_secret": { "type": "string", "minLength": 1 },
            "entra_token_secret": { "type": "string", "minLength": 1 },
            "entra_scope": { "type": "string", "minLength": 1 },
            "timeout_ms": { "type": "integer", "minimum": 1 },
            "allow_tools": { "type": "boolean" },
            "allow_structured_outputs": { "type": "boolean" },
            "allow_stateful_responses": { "type": "boolean" },
            "use_responses_api": { "type": "boolean" },
            "raw_headers": { "type": "object" }
        },
        "additionalProperties": false
    })
}

fn validate_https_endpoint(endpoint: &str) -> Result<(), AzureOpenAiError> {
    let parsed = Url::parse(endpoint)
        .map_err(|_| AzureOpenAiError::config("endpoint must be a valid absolute URL"))?;
    if parsed.scheme() != "https" {
        return Err(AzureOpenAiError::config(
            "endpoint must use https for the Azure OpenAI provider",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::AzureOpenAiConfig;

    #[test]
    fn validate_rejects_http_endpoint() {
        let mut config = AzureOpenAiConfig::new("http://example.openai.azure.com", "gpt-5", 5_000);
        config.api_key_secret = Some("secret".to_string());
        let err = config.validate().expect_err("http endpoint should fail");
        assert_eq!(
            err.to_string(),
            "endpoint must use https for the Azure OpenAI provider"
        );
    }
}
