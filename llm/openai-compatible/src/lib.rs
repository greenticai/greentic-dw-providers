#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Generic OpenAI-compatible provider built on top of the shared normalized LLM contract.

mod config;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{OpenAiCompatMode, OpenAiCompatibleConfig, openai_compatible_config_schema};
pub use errors::OpenAiCompatibleError;
pub use qa::{
    OpenAiCompatibleWizardQuestion, openai_compatible_wizard_qa, openai_compatible_wizard_questions,
};
pub use transport::{HttpOpenAiCompatibleTransport, OpenAiCompatibleTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    openai_compatible_llm_feature_profile, openai_compatible_llm_pack_manifest,
    openai_compatible_llm_provider_declaration, openai_compatible_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_request, parse_response};

/// Canonical provider slug for the generic compatibility backend.
pub const OPENAI_COMPATIBLE_PROVIDER_NAME: &str = "openai-compatible";

/// Generic OpenAI-compatible provider implementation.
pub struct OpenAiCompatibleProvider<T = HttpOpenAiCompatibleTransport> {
    config: OpenAiCompatibleConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl OpenAiCompatibleProvider<HttpOpenAiCompatibleTransport> {
    /// Creates a provider with the default HTTP transport.
    pub fn new(config: OpenAiCompatibleConfig) -> LlmResult<Self> {
        Self::with_transport(config, HttpOpenAiCompatibleTransport)
    }
}

impl<T> OpenAiCompatibleProvider<T>
where
    T: OpenAiCompatibleTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: OpenAiCompatibleConfig, transport: T) -> LlmResult<Self> {
        config.validate().map_err(greentic_dw_llm::LlmError::from)?;
        let features = config.feature_profile();
        Ok(Self {
            config,
            features,
            transport,
        })
    }

    /// Returns the canonical provider id.
    #[must_use]
    pub fn provider_id() -> String {
        openai_compatible_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        openai_compatible_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.openai-compatible")?;
        openai_compatible_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the default feature profile for this backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        openai_compatible_llm_feature_profile()
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &OpenAiCompatibleConfig {
        &self.config
    }
}

impl<T> LlmProvider for OpenAiCompatibleProvider<T>
where
    T: OpenAiCompatibleTransport,
{
    fn features(&self) -> &LlmProviderFeatures {
        &self.features
    }

    fn generate(&self, _tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse> {
        self.validate_request(&request)?;
        let payload = build_request(&self.config, &request)?;
        let response = self.transport.execute(&self.config, &payload)?;
        parse_response(&self.config, &response, &request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        HttpOpenAiCompatibleTransport, OpenAiCompatMode, OpenAiCompatibleConfig,
        OpenAiCompatibleProvider, OpenAiCompatibleTransport, openai_compatible_config_schema,
        openai_compatible_wizard_questions,
    };
    use greentic_dw_llm::{LlmProvider, LlmToolChoice, fixtures};
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::{Value, json};
    use std::sync::Mutex;

    struct RecordingTransport {
        payload: Mutex<Option<Value>>,
        response: Value,
    }

    impl RecordingTransport {
        fn new(response: Value) -> Self {
            Self {
                payload: Mutex::new(None),
                response,
            }
        }
    }

    impl OpenAiCompatibleTransport for RecordingTransport {
        fn execute(
            &self,
            _config: &OpenAiCompatibleConfig,
            payload: &Value,
        ) -> greentic_dw_llm::LlmResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-compatible").expect("tenant id"),
        )
    }

    fn responses_config() -> OpenAiCompatibleConfig {
        let mut config =
            OpenAiCompatibleConfig::new("http://localhost:11434/v1", "llama3.2", 10_000);
        config.compat_mode = OpenAiCompatMode::Responses;
        config.supports_structured_outputs = true;
        config
    }

    fn chat_config() -> OpenAiCompatibleConfig {
        let mut config = OpenAiCompatibleConfig::new("http://localhost:1234/v1", "qwen2.5", 10_000);
        config.compat_mode = OpenAiCompatMode::ChatCompletions;
        config.supports_structured_outputs = true;
        config
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = openai_compatible_config_schema();
        assert_eq!(
            schema["properties"]["compat_mode"]["enum"][0],
            json!("responses")
        );
        assert_eq!(
            schema["properties"]["compat_mode"]["enum"][1],
            json!("chat_completions")
        );
    }

    #[test]
    fn config_validation_rejects_empty_model() {
        let mut config = responses_config();
        config.model = String::new();
        let err = config.validate().expect_err("invalid config");
        assert_eq!(err.to_string(), "model must not be empty");
    }

    #[test]
    fn responses_mode_maps_text_requests() {
        let request = fixtures::text_request().expect("request");
        let provider = OpenAiCompatibleProvider::with_transport(
            responses_config(),
            RecordingTransport::new(json!({
                "id": "resp_1",
                "model": "llama3.2",
                "status": "completed",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{ "type": "output_text", "text": "hello from ollama" }]
                }]
            })),
        )
        .expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");
        assert_eq!(response.text_output(), "hello from ollama");
    }

    #[test]
    fn responses_mode_maps_structured_outputs() {
        let request = fixtures::structured_output_request().expect("request");
        let provider = OpenAiCompatibleProvider::with_transport(
            responses_config(),
            RecordingTransport::new(json!({
                "id": "resp_json",
                "model": "llama3.2",
                "status": "completed",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{ "type": "output_text", "text": "{\"message\":\"hi\"}" }]
                }]
            })),
        )
        .expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");
        assert_eq!(response.structured_output, Some(json!({"message":"hi"})));
    }

    #[test]
    fn chat_completions_mode_maps_tool_calls() {
        let request = fixtures::tool_request()
            .expect("request")
            .with_tool_choice(LlmToolChoice::Auto);
        let provider = OpenAiCompatibleProvider::with_transport(
            chat_config(),
            RecordingTransport::new(json!({
                "id": "chatcmpl_1",
                "model": "qwen2.5",
                "choices": [{
                    "finish_reason": "tool_calls",
                    "message": {
                        "role": "assistant",
                        "tool_calls": [{
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "lookup_weather",
                                "arguments": "{\"city\":\"Rotterdam\"}"
                            }
                        }]
                    }
                }],
                "usage": {
                    "prompt_tokens": 12,
                    "completion_tokens": 5,
                    "total_tokens": 17
                }
            })),
        )
        .expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "lookup_weather");
        assert_eq!(response.usage.expect("usage").total_tokens, 17);
    }

    #[test]
    fn compat_provider_rejects_stateful_requests_when_disabled() {
        let mut config = responses_config();
        config.supports_stateful_responses = false;
        let provider =
            OpenAiCompatibleProvider::with_transport(config, RecordingTransport::new(json!({})))
                .expect("provider");

        let request = fixtures::text_request()
            .expect("request")
            .with_conversation(greentic_dw_llm::LlmConversationState {
                conversation_id: None,
                response_id: Some("resp_prev".to_string()),
                provider_state: None,
            });
        let err = provider
            .generate(&tenant(), request)
            .expect_err("stateful request should fail");
        assert_eq!(err.kind, greentic_dw_llm::LlmErrorKind::UnsupportedFeature);
    }

    #[test]
    fn wizard_questions_include_compatibility_fields() {
        let questions = openai_compatible_wizard_questions();
        assert!(questions.iter().any(|question| question.key == "base_url"));
        assert!(
            questions
                .iter()
                .any(|question| question.key == "compat_mode")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "supports_structured_outputs")
        );
        assert!(questions.iter().any(|question| {
            question.key == "compat_mode" && question.default_value.as_deref() == Some("responses")
        }));
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(
            OpenAiCompatibleProvider::<HttpOpenAiCompatibleTransport>::provider_id(),
            "dw.llm.openai-compatible"
        );
        let manifest = OpenAiCompatibleProvider::<HttpOpenAiCompatibleTransport>::pack_manifest()
            .expect("pack manifest");
        assert_eq!(
            manifest.pack_id.to_string(),
            "greentic.dw.providers.llm.openai-compatible"
        );
    }
}
