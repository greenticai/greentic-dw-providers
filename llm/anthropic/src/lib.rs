#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Anthropic Claude provider built on top of the shared normalized LLM contract.

mod config;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{ANTHROPIC_DEFAULT_BASE_URL, AnthropicConfig, anthropic_config_schema};
pub use errors::AnthropicError;
pub use qa::{AnthropicWizardQuestion, anthropic_wizard_qa, anthropic_wizard_questions};
pub use transport::{AnthropicTransport, HttpAnthropicTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    anthropic_llm_feature_profile, anthropic_llm_pack_manifest, anthropic_llm_provider_declaration,
    anthropic_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_messages_request, parse_messages_response};

/// Canonical provider slug for the Anthropic backend.
pub const ANTHROPIC_PROVIDER_NAME: &str = "anthropic";

/// Anthropic provider implementation.
pub struct AnthropicProvider<T = HttpAnthropicTransport> {
    config: AnthropicConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl AnthropicProvider<HttpAnthropicTransport> {
    /// Creates a provider with the default HTTP transport.
    pub fn new(config: AnthropicConfig) -> LlmResult<Self> {
        Self::with_transport(config, HttpAnthropicTransport)
    }
}

impl<T> AnthropicProvider<T>
where
    T: AnthropicTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: AnthropicConfig, transport: T) -> LlmResult<Self> {
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
        anthropic_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        anthropic_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.anthropic")?;
        anthropic_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &AnthropicConfig {
        &self.config
    }

    /// Returns the default feature profile for the backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        anthropic_llm_feature_profile()
    }
}

impl<T> LlmProvider for AnthropicProvider<T>
where
    T: AnthropicTransport,
{
    fn features(&self) -> &LlmProviderFeatures {
        &self.features
    }

    fn generate(&self, _tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse> {
        self.validate_request(&request)?;
        let payload = build_messages_request(&self.config, &request)?;
        let response = self.transport.create_message(&self.config, &payload)?;
        parse_messages_response(&response, &request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        AnthropicConfig, AnthropicProvider, AnthropicTransport, anthropic_config_schema,
        anthropic_wizard_questions,
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

        fn payload(&self) -> Value {
            self.payload
                .lock()
                .expect("payload mutex")
                .clone()
                .expect("payload")
        }
    }

    impl AnthropicTransport for RecordingTransport {
        fn create_message(
            &self,
            _config: &AnthropicConfig,
            payload: &Value,
        ) -> greentic_dw_llm::LlmResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-anthropic").expect("tenant id"),
        )
    }

    fn config() -> AnthropicConfig {
        AnthropicConfig::new("sk-ant", "claude-3-5-sonnet-latest", 1024, 10_000)
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = anthropic_config_schema();
        assert_eq!(schema["properties"]["max_tokens"]["minimum"], json!(1));
        assert_eq!(
            schema["properties"]["allow_thinking"]["type"],
            json!("boolean")
        );
    }

    #[test]
    fn config_validation_rejects_empty_model() {
        let mut config = config();
        config.model = String::new();
        let err = config.validate().expect_err("invalid config");
        assert_eq!(err.to_string(), "model must not be empty");
    }

    #[test]
    fn text_request_maps_to_messages_payload() {
        let transport = RecordingTransport::new(json!({
            "id": "msg_1",
            "model": "claude-3-5-sonnet-latest",
            "role": "assistant",
            "content": [{ "type": "text", "text": "hello claude" }],
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 10, "output_tokens": 4 }
        }));
        let provider = AnthropicProvider::with_transport(config(), transport).expect("provider");

        let response = provider
            .generate(&tenant(), fixtures::text_request().expect("request"))
            .expect("response");
        assert_eq!(response.text_output(), "hello claude");
    }

    #[test]
    fn tool_request_maps_to_tool_use_output() {
        let transport = RecordingTransport::new(json!({
            "id": "msg_tool",
            "model": "claude-3-5-sonnet-latest",
            "role": "assistant",
            "content": [{
                "type": "tool_use",
                "id": "toolu_1",
                "name": "lookup_weather",
                "input": { "city": "Rotterdam" }
            }],
            "stop_reason": "tool_use",
            "usage": { "input_tokens": 12, "output_tokens": 5 }
        }));
        let provider = AnthropicProvider::with_transport(config(), transport).expect("provider");

        let response = provider
            .generate(&tenant(), fixtures::tool_request().expect("request"))
            .expect("response");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "lookup_weather");
        assert_eq!(
            response.tool_calls[0].arguments,
            json!({ "city": "Rotterdam" })
        );
    }

    #[test]
    fn structured_output_request_maps_to_synthetic_tool() {
        let transport = RecordingTransport::new(json!({
            "id": "msg_structured",
            "model": "claude-3-5-sonnet-latest",
            "role": "assistant",
            "content": [{
                "type": "tool_use",
                "id": "toolu_structured",
                "name": "schema_target",
                "input": { "message": "hi" }
            }],
            "stop_reason": "tool_use",
            "usage": { "input_tokens": 12, "output_tokens": 5 }
        }));
        let provider = AnthropicProvider::with_transport(config(), transport).expect("provider");
        let mut request = fixtures::structured_output_request().expect("request");
        request.structured_output.as_mut().expect("schema").name = "schema_target".to_string();

        let response = provider.generate(&tenant(), request).expect("response");
        assert_eq!(response.structured_output, Some(json!({ "message": "hi" })));
    }

    #[test]
    fn request_payload_carries_tools_and_structured_output_force() {
        let mut request = fixtures::structured_output_request().expect("request");
        request.structured_output.as_mut().expect("schema").name = "schema_target".to_string();
        let recording = RecordingTransport::new(json!({
            "id": "msg_payload",
            "model": "claude-3-5-sonnet-latest",
            "role": "assistant",
            "content": [],
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 1, "output_tokens": 1 }
        }));
        let provider = AnthropicProvider::with_transport(config(), recording).expect("provider");
        let _ = provider.generate(&tenant(), request).expect("response");
        let payload = provider.transport.payload();
        assert_eq!(payload["tool_choice"]["type"], json!("tool"));
        assert_eq!(payload["tool_choice"]["name"], json!("schema_target"));
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(
            AnthropicProvider::<super::HttpAnthropicTransport>::provider_id(),
            "dw.llm.anthropic"
        );
        let decl = AnthropicProvider::<super::HttpAnthropicTransport>::provider_decl();
        assert_eq!(decl.provider_type, "dw.llm.anthropic");
        let manifest = AnthropicProvider::<super::HttpAnthropicTransport>::pack_manifest()
            .expect("pack manifest");
        assert_eq!(
            manifest.pack_id.to_string(),
            "greentic.dw.providers.llm.anthropic"
        );
    }

    #[test]
    fn wizard_questions_include_thinking_fields() {
        let questions = anthropic_wizard_questions();
        assert!(questions.iter().any(|q| q.key == "allow_thinking"));
        assert!(questions.iter().any(|q| q.key == "thinking_budget_tokens"));
        assert!(questions.iter().any(|q| {
            q.key == "thinking_budget_tokens"
                && matches!(
                    q.visibility,
                    greentic_dw_providers_common::LlmWizardVisibility::Equals { .. }
                )
        }));
    }

    #[test]
    fn feature_profile_exposes_enterprise_auth_without_stateful_conversation() {
        let provider = AnthropicProvider::with_transport(
            config(),
            RecordingTransport::new(json!({
                "id": "msg_1",
                "model": "claude-3-5-sonnet-latest",
                "role": "assistant",
                "content": [],
                "stop_reason": "end_turn",
                "usage": { "input_tokens": 1, "output_tokens": 1 }
            })),
        )
        .expect("provider");

        assert!(provider.features().chat);
        assert!(provider.features().tool_calling);
        assert!(provider.features().structured_outputs);
        assert!(provider.features().enterprise_auth);
        assert!(!provider.features().stateful_conversation);
    }

    #[test]
    fn config_validation_requires_thinking_budget_when_enabled() {
        let mut config = config();
        config.allow_thinking = true;
        let err = config.validate().expect_err("invalid config");
        assert_eq!(
            err.to_string(),
            "thinking_budget_tokens must be set when allow_thinking is enabled"
        );
    }

    #[test]
    fn thinking_enabled_adds_thinking_block_to_payload() {
        let mut config = config();
        config.allow_thinking = true;
        config.thinking_budget_tokens = Some(256);
        let recording = RecordingTransport::new(json!({
            "id": "msg_payload",
            "model": "claude-3-5-sonnet-latest",
            "role": "assistant",
            "content": [],
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 1, "output_tokens": 1 }
        }));
        let provider = AnthropicProvider::with_transport(config, recording).expect("provider");
        let _ = provider
            .generate(&tenant(), fixtures::text_request().expect("request"))
            .expect("response");
        let payload = provider.transport.payload();
        assert_eq!(payload["thinking"]["type"], json!("enabled"));
        assert_eq!(payload["thinking"]["budget_tokens"], json!(256));
    }

    #[test]
    fn messages_payload_uses_any_for_required_tool_choice() {
        let recording = RecordingTransport::new(json!({
            "id": "msg_payload",
            "model": "claude-3-5-sonnet-latest",
            "role": "assistant",
            "content": [],
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 1, "output_tokens": 1 }
        }));
        let provider = AnthropicProvider::with_transport(config(), recording).expect("provider");
        let request = fixtures::tool_request()
            .expect("request")
            .with_tool_choice(LlmToolChoice::Required);
        let _ = provider.generate(&tenant(), request).expect("response");
        let payload = provider.transport.payload();
        assert_eq!(payload["tool_choice"]["type"], json!("any"));
    }

    #[test]
    fn http_error_mapping_prefixes_messages_api_failures() {
        let err = super::mapping::map_http_error(429, r#"{"error":{"message":"rate limit"}}"#);
        assert_eq!(err.kind, greentic_dw_llm::LlmErrorKind::RateLimited);
        assert!(err.retryable);
        assert!(
            err.message
                .contains("Anthropic Messages API request failed")
        );
    }
}
