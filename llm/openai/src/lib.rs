#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Native OpenAI provider built on top of the shared normalized LLM contract.

mod config;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{
    OPENAI_DEFAULT_BASE_URL, OpenAiConfig, OpenAiReasoningProfile, openai_config_schema,
};
pub use errors::OpenAiError;
pub use qa::{OpenAiWizardQuestion, openai_wizard_qa, openai_wizard_questions};
pub use transport::{HttpOpenAiTransport, OpenAiTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    openai_llm_feature_profile, openai_llm_pack_manifest, openai_llm_provider_declaration,
    openai_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_responses_request, parse_responses_response};

/// Canonical provider slug for the native OpenAI backend.
pub const OPENAI_PROVIDER_NAME: &str = "openai";

/// Native OpenAI provider implementation.
pub struct OpenAiProvider<T = HttpOpenAiTransport> {
    config: OpenAiConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl OpenAiProvider<HttpOpenAiTransport> {
    /// Creates a provider with the default HTTP transport.
    pub fn new(config: OpenAiConfig) -> LlmResult<Self> {
        Self::with_transport(config, HttpOpenAiTransport)
    }
}

impl<T> OpenAiProvider<T>
where
    T: OpenAiTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: OpenAiConfig, transport: T) -> LlmResult<Self> {
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
        openai_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        openai_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.openai")?;
        openai_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &OpenAiConfig {
        &self.config
    }

    /// Returns the default feature profile for the backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        openai_llm_feature_profile()
    }
}

impl<T> LlmProvider for OpenAiProvider<T>
where
    T: OpenAiTransport,
{
    fn features(&self) -> &LlmProviderFeatures {
        &self.features
    }

    fn generate(&self, _tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse> {
        self.validate_request(&request)?;
        let payload = build_responses_request(&self.config, &request)?;
        let response = self.transport.create_response(&self.config, &payload)?;
        parse_responses_response(&response, &request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        OPENAI_PROVIDER_NAME, OpenAiConfig, OpenAiProvider, OpenAiReasoningProfile,
        OpenAiTransport, openai_config_schema, openai_wizard_questions,
    };
    use greentic_dw_llm::{
        LlmMessage, LlmMessageRole, LlmProvider, LlmStructuredOutputSpec, LlmToolChoice,
        LlmToolSpec, fixtures,
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

    impl OpenAiTransport for RecordingTransport {
        fn create_response(
            &self,
            _config: &OpenAiConfig,
            payload: &Value,
        ) -> greentic_dw_llm::LlmResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-openai").expect("tenant id"),
        )
    }

    fn config() -> OpenAiConfig {
        let mut config = OpenAiConfig::new("sk-test", "gpt-4.1-mini", 10_000);
        config.reasoning_profile = Some(OpenAiReasoningProfile::Medium);
        config
    }

    #[test]
    fn config_validation_rejects_empty_model() {
        let mut config = OpenAiConfig::new("sk-test", "", 10_000);
        config.model = String::new();
        let err = config.validate().expect_err("invalid config");
        assert_eq!(err.to_string(), "model must not be empty");
    }

    #[test]
    fn provider_exposes_expected_feature_flags() {
        let provider = OpenAiProvider::with_transport(
            config(),
            RecordingTransport::new(json!({ "output": [] })),
        )
        .expect("provider");

        assert!(provider.features().chat);
        assert!(provider.features().streaming);
        assert!(provider.features().stateful_conversation);
    }

    #[test]
    fn text_request_maps_to_responses_payload() {
        let request = fixtures::text_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "resp_1",
            "model": "gpt-4.1-mini",
            "status": "completed",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "Hello." }]
            }]
        }));
        let provider = OpenAiProvider::with_transport(config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.text_output(), "Hello.");
    }

    #[test]
    fn structured_output_request_maps_to_json_schema_format() {
        let request = fixtures::structured_output_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "resp_structured",
            "model": "gpt-4.1-mini",
            "status": "completed",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "{\"message\":\"hi\"}" }]
            }]
        }));
        let provider = OpenAiProvider::with_transport(config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.structured_output, Some(json!({ "message": "hi" })));
    }

    #[test]
    fn tool_request_maps_to_function_tools_and_tool_calls() {
        let request = fixtures::tool_request().expect("request");
        let transport = RecordingTransport::new(json!({
            "id": "resp_tool",
            "model": "gpt-4.1-mini",
            "status": "completed",
            "output": [{
                "type": "function_call",
                "id": "fc_1",
                "call_id": "call_1",
                "name": "lookup_weather",
                "arguments": "{\"city\":\"Rotterdam\"}"
            }]
        }));
        let provider = OpenAiProvider::with_transport(config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "lookup_weather");
        assert_eq!(
            response.tool_calls[0].arguments,
            json!({ "city": "Rotterdam" })
        );
    }

    #[test]
    fn request_payload_includes_previous_response_id_and_forced_tool_choice() {
        let request = fixtures::tool_request()
            .expect("request")
            .with_tool_choice(LlmToolChoice::Tool("lookup_weather".to_string()))
            .with_conversation(greentic_dw_llm::LlmConversationState {
                conversation_id: None,
                response_id: Some("resp_prev".to_string()),
                provider_state: None,
            })
            .with_stream(true);
        let transport = RecordingTransport::new(json!({
            "id": "resp_next",
            "model": "gpt-4.1-mini",
            "status": "completed",
            "output": []
        }));
        let provider = OpenAiProvider::with_transport(config(), transport).expect("provider");
        let transport = &provider.transport;

        provider.generate(&tenant(), request).expect("response");
        let payload = transport.payload();

        assert_eq!(payload["previous_response_id"], json!("resp_prev"));
        assert_eq!(payload["tool_choice"]["name"], json!("lookup_weather"));
        assert_eq!(payload["stream"], json!(true));
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(
            OpenAiProvider::<super::HttpOpenAiTransport>::provider_id(),
            "dw.llm.openai"
        );
        let decl = OpenAiProvider::<super::HttpOpenAiTransport>::provider_decl();
        assert_eq!(decl.provider_type, format!("dw.llm.{OPENAI_PROVIDER_NAME}"));
        let manifest =
            OpenAiProvider::<super::HttpOpenAiTransport>::pack_manifest().expect("pack manifest");
        assert_eq!(
            manifest.pack_id.to_string(),
            "greentic.dw.providers.llm.openai"
        );
    }

    #[test]
    fn wizard_questions_include_core_openai_fields() {
        let questions = openai_wizard_questions();
        assert!(
            questions
                .iter()
                .any(|question| question.key == "api_key_secret")
        );
        assert!(questions.iter().any(|question| question.key == "model"));
        assert!(
            questions
                .iter()
                .any(|question| question.key == "allow_structured_outputs")
        );
        assert!(questions.iter().any(|question| {
            question.prompt_key == "wizard.llm.shared.timeout_ms"
                && question.default_value.as_deref() == Some("60000")
        }));
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = openai_config_schema();
        assert_eq!(schema["type"], json!("object"));
        assert_eq!(
            schema["properties"]["api_key_secret"]["type"],
            json!("string")
        );
        assert_eq!(schema["properties"]["model"]["type"], json!("string"));
        assert_eq!(
            schema["properties"]["reasoning_profile"]["enum"][1],
            json!("medium")
        );
    }

    #[test]
    fn provider_respects_feature_validation() {
        let mut config = config();
        config.allow_tools = false;
        let provider = OpenAiProvider::with_transport(
            config,
            RecordingTransport::new(json!({ "output": [] })),
        )
        .expect("provider");

        let err = provider
            .generate(&tenant(), fixtures::tool_request().expect("request"))
            .expect_err("tool support should be rejected");
        assert_eq!(err.kind, greentic_dw_llm::LlmErrorKind::UnsupportedFeature);
    }

    #[test]
    fn response_parser_handles_manual_fixture_shape() {
        let request = greentic_dw_llm::LlmRequest::new(
            "req-manual",
            vec![LlmMessage::text(LlmMessageRole::User, "hello").expect("message")],
        )
        .expect("request")
        .with_structured_output(
            LlmStructuredOutputSpec::new("answer", json!({ "type": "object" })).expect("schema"),
        )
        .with_tool(LlmToolSpec::new("lookup", json!({ "type": "object" })).expect("tool"));
        let transport = RecordingTransport::new(json!({
            "id": "resp_manual",
            "model": "gpt-4.1-mini",
            "status": "completed",
            "usage": {
                "input_tokens": 11,
                "output_tokens": 7,
                "total_tokens": 18
            },
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "{\"ok\":true}" }]
            }]
        }));
        let provider = OpenAiProvider::with_transport(config(), transport).expect("provider");

        let response = provider.generate(&tenant(), request).expect("response");

        assert_eq!(response.usage.expect("usage").total_tokens, 18);
        assert_eq!(response.structured_output, Some(json!({ "ok": true })));
    }
}
