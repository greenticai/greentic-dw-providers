#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Gemini provider built on top of the shared normalized LLM contract.

mod config;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{GEMINI_DEFAULT_BASE_URL, GeminiConfig, gemini_config_schema};
pub use errors::GeminiError;
pub use qa::{GeminiWizardQuestion, gemini_wizard_qa, gemini_wizard_questions};
pub use transport::{GeminiTransport, HttpGeminiTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    gemini_llm_feature_profile, gemini_llm_pack_manifest, gemini_llm_provider_declaration,
    gemini_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_generate_content_request, parse_generate_content_response};

/// Canonical provider slug for the Gemini backend.
pub const GEMINI_PROVIDER_NAME: &str = "gemini";

/// Gemini provider implementation.
pub struct GeminiProvider<T = HttpGeminiTransport> {
    config: GeminiConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl GeminiProvider<HttpGeminiTransport> {
    /// Creates a provider with the default HTTP transport.
    pub fn new(config: GeminiConfig) -> LlmResult<Self> {
        Self::with_transport(config, HttpGeminiTransport)
    }
}

impl<T> GeminiProvider<T>
where
    T: GeminiTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: GeminiConfig, transport: T) -> LlmResult<Self> {
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
        gemini_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        gemini_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.gemini")?;
        gemini_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &GeminiConfig {
        &self.config
    }

    /// Returns the default feature profile for the backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        gemini_llm_feature_profile()
    }
}

impl<T> LlmProvider for GeminiProvider<T>
where
    T: GeminiTransport,
{
    fn features(&self) -> &LlmProviderFeatures {
        &self.features
    }

    fn generate(&self, _tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse> {
        self.validate_request(&request)?;
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.config.model.clone());
        let payload = build_generate_content_request(&self.config, &request)?;
        let response = self
            .transport
            .generate_content(&self.config, &model, &payload)?;
        parse_generate_content_response(&response, &request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        GeminiConfig, GeminiProvider, GeminiTransport, gemini_config_schema,
        gemini_wizard_questions,
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

    impl GeminiTransport for RecordingTransport {
        fn generate_content(
            &self,
            _config: &GeminiConfig,
            _model: &str,
            payload: &Value,
        ) -> greentic_dw_llm::LlmResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-gemini").expect("tenant id"),
        )
    }

    fn config() -> GeminiConfig {
        GeminiConfig::new("gemini-key", "gemini-2.5-flash", 10_000)
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = gemini_config_schema();
        assert_eq!(schema["properties"]["timeout_ms"]["minimum"], json!(1));
        assert_eq!(
            schema["properties"]["safety_profile"]["type"],
            json!("string")
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
    fn text_request_maps_to_generate_content_payload() {
        let transport = RecordingTransport::new(json!({
            "candidates": [{
                "content": { "role": "model", "parts": [{ "text": "hello gemini" }] },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 4,
                "totalTokenCount": 14
            },
            "modelVersion": "gemini-2.5-flash"
        }));
        let provider = GeminiProvider::with_transport(config(), transport).expect("provider");

        let response = provider
            .generate(&tenant(), fixtures::text_request().expect("request"))
            .expect("response");
        assert_eq!(response.text_output(), "hello gemini");
        assert_eq!(response.model.as_deref(), Some("gemini-2.5-flash"));
    }

    #[test]
    fn tool_request_maps_to_function_calls() {
        let transport = RecordingTransport::new(json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "functionCall": {
                            "name": "lookup_weather",
                            "args": { "city": "Rotterdam" }
                        }
                    }]
                },
                "finishReason": "STOP"
            }]
        }));
        let provider = GeminiProvider::with_transport(config(), transport).expect("provider");

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
    fn structured_output_request_maps_to_response_json_schema() {
        let transport = RecordingTransport::new(json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{ "text": "{\"message\":\"hi\"}" }]
                },
                "finishReason": "STOP"
            }]
        }));
        let provider = GeminiProvider::with_transport(config(), transport).expect("provider");

        let response = provider
            .generate(
                &tenant(),
                fixtures::structured_output_request().expect("request"),
            )
            .expect("response");
        assert_eq!(response.structured_output, Some(json!({ "message": "hi" })));
    }

    #[test]
    fn required_tool_choice_maps_to_any_mode() {
        let request = fixtures::tool_request()
            .expect("request")
            .with_tool_choice(LlmToolChoice::Required);
        let transport = RecordingTransport::new(json!({
            "candidates": [{
                "content": { "role": "model", "parts": [] },
                "finishReason": "STOP"
            }]
        }));
        let provider = GeminiProvider::with_transport(config(), transport).expect("provider");
        provider.generate(&tenant(), request).expect("response");

        let payload = provider.transport.payload();
        assert_eq!(
            payload["toolConfig"]["functionCallingConfig"]["mode"],
            json!("ANY")
        );
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(
            GeminiProvider::<super::HttpGeminiTransport>::provider_id(),
            "dw.llm.gemini"
        );
        let declaration = GeminiProvider::<super::HttpGeminiTransport>::provider_decl();
        assert_eq!(declaration.provider_type, "dw.llm.gemini");
        let manifest =
            GeminiProvider::<super::HttpGeminiTransport>::pack_manifest().expect("pack manifest");
        assert_eq!(manifest.capabilities[0].name, "greentic.cap.llm");
    }

    #[test]
    fn wizard_questions_include_gemini_fields() {
        let questions = gemini_wizard_questions();
        assert!(
            questions
                .iter()
                .any(|question| question.key == "api_key_secret")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "safety_profile")
        );
        assert!(questions.iter().any(|question| {
            question.prompt_key == "wizard.llm.shared.allow_structured_outputs"
                && question.default_value.as_deref() == Some("true")
        }));
    }

    #[test]
    fn feature_profile_exposes_structured_output_without_stateful_conversation() {
        let features = GeminiProvider::<super::HttpGeminiTransport>::default_features();
        assert!(features.chat);
        assert!(features.structured_outputs);
        assert!(features.tool_calling);
        assert!(!features.stateful_conversation);
        assert!(!features.local_self_hosted);
    }

    #[test]
    fn request_payload_carries_generation_config_for_structured_output() {
        let transport = RecordingTransport::new(json!({
            "candidates": [{
                "content": { "role": "model", "parts": [{ "text": "{\"message\":\"hi\"}" }] },
                "finishReason": "STOP"
            }]
        }));
        let provider = GeminiProvider::with_transport(config(), transport).expect("provider");
        provider
            .generate(
                &tenant(),
                fixtures::structured_output_request().expect("request"),
            )
            .expect("response");

        let payload = provider.transport.payload();
        assert_eq!(
            payload["generationConfig"]["responseMimeType"],
            json!("application/json")
        );
        assert!(payload["generationConfig"]["responseJsonSchema"].is_object());
    }
}
