#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Azure OpenAI provider built on top of the shared normalized LLM contract.

mod config;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{
    AZURE_OPENAI_DEFAULT_API_VERSION, AZURE_OPENAI_DEFAULT_ENTRA_SCOPE, AzureOpenAiAuthMode,
    AzureOpenAiConfig, azure_openai_config_schema,
};
pub use errors::AzureOpenAiError;
pub use qa::{AzureOpenAiWizardQuestion, azure_openai_wizard_qa, azure_openai_wizard_questions};
pub use transport::{AzureOpenAiTransport, HttpAzureOpenAiTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    azure_openai_llm_feature_profile, azure_openai_llm_pack_manifest,
    azure_openai_llm_provider_declaration, azure_openai_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_request, parse_response};

/// Canonical provider slug for the Azure OpenAI backend.
pub const AZURE_OPENAI_PROVIDER_NAME: &str = "azure-openai";

/// Azure OpenAI provider implementation.
pub struct AzureOpenAiProvider<T = HttpAzureOpenAiTransport> {
    config: AzureOpenAiConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl AzureOpenAiProvider<HttpAzureOpenAiTransport> {
    /// Creates a provider with the default HTTP transport.
    pub fn new(config: AzureOpenAiConfig) -> LlmResult<Self> {
        Self::with_transport(config, HttpAzureOpenAiTransport)
    }
}

impl<T> AzureOpenAiProvider<T>
where
    T: AzureOpenAiTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: AzureOpenAiConfig, transport: T) -> LlmResult<Self> {
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
        azure_openai_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        azure_openai_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.azure-openai")?;
        azure_openai_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &AzureOpenAiConfig {
        &self.config
    }

    /// Returns the default family feature profile for the backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        azure_openai_llm_feature_profile()
    }
}

impl<T> LlmProvider for AzureOpenAiProvider<T>
where
    T: AzureOpenAiTransport,
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
        AZURE_OPENAI_PROVIDER_NAME, AzureOpenAiAuthMode, AzureOpenAiConfig, AzureOpenAiProvider,
        AzureOpenAiTransport, azure_openai_config_schema, azure_openai_wizard_questions,
    };
    use greentic_dw_llm::{
        LlmProvider, LlmStructuredOutputSpec, LlmToolChoice, LlmToolSpec, fixtures,
    };
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

        fn payload(&self) -> Value {
            self.payload
                .lock()
                .expect("payload mutex")
                .clone()
                .expect("captured payload")
        }
    }

    impl AzureOpenAiTransport for RecordingTransport {
        fn execute(
            &self,
            _config: &AzureOpenAiConfig,
            payload: &Value,
        ) -> greentic_dw_llm::LlmResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-azure-openai").expect("tenant id"),
        )
    }

    fn sample_config() -> AzureOpenAiConfig {
        let mut config = AzureOpenAiConfig::new(
            "https://example.openai.azure.com",
            "gpt-4.1-mini-deploy",
            10_000,
        );
        config.api_key_secret = Some("azure-key".to_string());
        config
    }

    #[test]
    fn config_validation_rejects_missing_auth_credentials() {
        let config = AzureOpenAiConfig::new(
            "https://example.openai.azure.com",
            "gpt-4.1-mini-deploy",
            10_000,
        );
        let err = config.validate().expect_err("invalid config");
        assert_eq!(
            err.to_string(),
            "api_key_secret is required for api_key auth"
        );

        let mut config = sample_config();
        config.auth_mode = AzureOpenAiAuthMode::EntraId;
        config.api_key_secret = None;
        config.entra_token_secret = None;
        let err = config.validate().expect_err("invalid entra config");
        assert_eq!(
            err.to_string(),
            "entra_token_secret is required for entra_id auth"
        );
    }

    #[test]
    fn provider_exposes_expected_feature_flags() {
        let provider = AzureOpenAiProvider::with_transport(
            sample_config(),
            RecordingTransport::new(json!({ "output": [] })),
        )
        .expect("provider");

        assert!(provider.features().chat);
        assert!(provider.features().streaming);
        assert!(provider.features().stateful_conversation);
        assert!(provider.features().enterprise_auth);
    }

    #[test]
    fn text_request_maps_to_responses_payload_using_deployment() {
        let request = fixtures::text_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "resp_1",
            "model": "gpt-4.1-mini-deploy",
            "status": "completed",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "Hello from Azure." }]
            }]
        }));
        let provider =
            AzureOpenAiProvider::with_transport(sample_config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.text_output(), "Hello from Azure.");
    }

    #[test]
    fn structured_output_request_maps_to_json_schema_format() {
        let request = fixtures::structured_output_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "resp_structured",
            "model": "gpt-4.1-mini-deploy",
            "status": "completed",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "{\"message\":\"hi\"}" }]
            }]
        }));
        let provider =
            AzureOpenAiProvider::with_transport(sample_config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.structured_output, Some(json!({ "message": "hi" })));
    }

    #[test]
    fn tool_request_maps_to_function_tools_and_tool_calls() {
        let request = fixtures::tool_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "resp_tool",
            "model": "gpt-4.1-mini-deploy",
            "status": "completed",
            "output": [{
                "type": "function_call",
                "id": "fc_1",
                "call_id": "call_1",
                "name": "lookup_weather",
                "arguments": "{\"city\":\"Rotterdam\"}"
            }]
        }));
        let provider =
            AzureOpenAiProvider::with_transport(sample_config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "lookup_weather");
    }

    #[test]
    fn request_payload_includes_previous_response_id_for_stateful_responses() {
        let request = fixtures::tool_request()
            .expect("request")
            .with_conversation(greentic_dw_llm::LlmConversationState {
                conversation_id: Some("conv-1".to_string()),
                response_id: Some("resp_prev".to_string()),
                provider_state: None,
            });
        let transport = RecordingTransport::new(json!({
            "id": "resp_next",
            "model": "gpt-4.1-mini-deploy",
            "status": "completed",
            "output": []
        }));
        let provider =
            AzureOpenAiProvider::with_transport(sample_config(), transport).expect("provider");
        let response = provider.generate(&tenant(), request).expect("response");
        let payload = provider.transport.payload();

        assert_eq!(response.response_id.as_deref(), Some("resp_next"));
        assert_eq!(payload["previous_response_id"], json!("resp_prev"));
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(AZURE_OPENAI_PROVIDER_NAME, "azure-openai");
        assert_eq!(
            AzureOpenAiProvider::<super::HttpAzureOpenAiTransport>::provider_id(),
            "dw.llm.azure-openai"
        );
        assert_eq!(
            AzureOpenAiProvider::<super::HttpAzureOpenAiTransport>::provider_decl().provider_type,
            "dw.llm.azure-openai"
        );
        assert_eq!(
            AzureOpenAiProvider::<super::HttpAzureOpenAiTransport>::pack_manifest()
                .expect("pack manifest")
                .pack_id
                .to_string(),
            "greentic.dw.providers.llm.azure-openai"
        );
    }

    #[test]
    fn feature_profile_respects_responses_api_stateful_toggle() {
        let mut config = sample_config();
        config.use_responses_api = false;
        config.allow_stateful_responses = false;
        let provider = AzureOpenAiProvider::with_transport(
            config,
            RecordingTransport::new(json!({ "choices": [] })),
        )
        .expect("provider");
        assert!(!provider.features().stateful_conversation);
        assert!(provider.features().enterprise_auth);
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = azure_openai_config_schema();
        let required = schema["required"].as_array().expect("required array");

        assert!(required.contains(&json!("endpoint")));
        assert!(required.contains(&json!("deployment")));
        assert!(required.contains(&json!("auth_mode")));
        assert!(required.contains(&json!("use_responses_api")));
        assert_eq!(
            schema["properties"]["auth_mode"]["enum"],
            json!(["api_key", "entra_id"])
        );
    }

    #[test]
    fn wizard_questions_include_azure_specific_fields() {
        let questions = azure_openai_wizard_questions();

        assert!(questions.iter().any(|question| question.key == "endpoint"));
        assert!(
            questions
                .iter()
                .any(|question| question.key == "deployment")
        );
        assert!(questions.iter().any(|question| question.key == "auth_mode"));
        assert!(
            questions
                .iter()
                .any(|question| question.key == "use_responses_api")
        );
    }

    #[test]
    fn chat_completions_mode_maps_tool_calls() {
        let request = fixtures::tool_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "chatcmpl_1",
            "model": "gpt-4.1-mini-deploy",
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
            }]
        }));
        let mut config = sample_config();
        config.use_responses_api = false;
        config.allow_stateful_responses = false;
        let provider = AzureOpenAiProvider::with_transport(config, transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "lookup_weather");
    }

    #[test]
    fn chat_completions_mode_structured_outputs_still_map() {
        let request = fixtures::text_request()
            .expect("request")
            .with_structured_output(
                LlmStructuredOutputSpec::new(
                    "greeting",
                    json!({
                        "type": "object",
                        "properties": { "message": { "type": "string" } },
                        "required": ["message"]
                    }),
                )
                .expect("structured output"),
            );
        let transport = RecordingTransport::new(json!({
            "id": "chatcmpl_2",
            "model": "gpt-4.1-mini-deploy",
            "choices": [{
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": "{\"message\":\"hi\"}"
                }
            }]
        }));
        let mut config = sample_config();
        config.use_responses_api = false;
        config.allow_stateful_responses = false;
        let provider = AzureOpenAiProvider::with_transport(config, transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.structured_output, Some(json!({ "message": "hi" })));
    }

    #[test]
    fn forced_tool_choice_maps_into_responses_payload() {
        let request = fixtures::text_request()
            .expect("request")
            .with_tool(
                LlmToolSpec::new(
                    "lookup_weather",
                    json!({
                        "type": "object",
                        "properties": { "city": { "type": "string" } },
                        "required": ["city"]
                    }),
                )
                .expect("tool spec"),
            )
            .with_tool_choice(LlmToolChoice::Tool("lookup_weather".to_string()));
        let transport = RecordingTransport::new(json!({
            "id": "resp_tool_choice",
            "model": "gpt-4.1-mini-deploy",
            "status": "completed",
            "output": []
        }));
        let provider =
            AzureOpenAiProvider::with_transport(sample_config(), transport).expect("provider");
        let _ = provider.generate(&tenant(), request).expect("response");
        let payload = provider.transport.payload();

        assert_eq!(payload["tool_choice"]["type"], json!("function"));
        assert_eq!(payload["tool_choice"]["name"], json!("lookup_weather"));
    }
}
