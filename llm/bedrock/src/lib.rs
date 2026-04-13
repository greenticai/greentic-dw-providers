#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Amazon Bedrock provider built on top of the shared normalized LLM contract.

mod config;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{BedrockAuthMode, BedrockConfig, bedrock_config_schema};
pub use errors::BedrockError;
pub use qa::{BedrockWizardQuestion, bedrock_wizard_qa, bedrock_wizard_questions};
pub use transport::{AwsSdkBedrockTransport, BedrockTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    bedrock_llm_feature_profile, bedrock_llm_pack_manifest, bedrock_llm_provider_declaration,
    bedrock_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_converse_request, parse_converse_response};

/// Canonical provider slug for the Bedrock backend.
pub const BEDROCK_PROVIDER_NAME: &str = "bedrock";

/// Bedrock provider implementation.
pub struct BedrockProvider<T = AwsSdkBedrockTransport> {
    config: BedrockConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl BedrockProvider<AwsSdkBedrockTransport> {
    /// Creates a provider with the default AWS SDK transport.
    pub fn new(config: BedrockConfig) -> LlmResult<Self> {
        Self::with_transport(config, AwsSdkBedrockTransport)
    }
}

impl<T> BedrockProvider<T>
where
    T: BedrockTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: BedrockConfig, transport: T) -> LlmResult<Self> {
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
        bedrock_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        bedrock_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.bedrock")?;
        bedrock_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &BedrockConfig {
        &self.config
    }

    /// Returns the default feature profile for the backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        bedrock_llm_feature_profile()
    }
}

impl<T> LlmProvider for BedrockProvider<T>
where
    T: BedrockTransport,
{
    fn features(&self) -> &LlmProviderFeatures {
        &self.features
    }

    fn generate(&self, _tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse> {
        self.validate_request(&request)?;
        let model_id = request
            .model
            .clone()
            .unwrap_or_else(|| self.config.model_id.clone());
        let converse_request = build_converse_request(&request)?;
        let response = if converse_request.stream && converse_request.tools.is_empty() {
            self.transport
                .converse_stream(&self.config, &model_id, &converse_request)?
        } else {
            self.transport
                .converse(&self.config, &model_id, &converse_request)?
        };
        Ok(parse_converse_response(response))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        BedrockAuthMode, BedrockConfig, BedrockProvider, BedrockTransport, bedrock_config_schema,
        bedrock_wizard_questions,
    };
    use crate::mapping::{
        BedrockContentPart, BedrockConverseRequest, BedrockConverseResponse, BedrockMessage,
        BedrockMessageRole,
    };
    use greentic_dw_llm::{LlmFinishReason, LlmProvider, LlmToolChoice, LlmUsage, fixtures};
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;
    use std::sync::Mutex;

    struct RecordingTransport {
        request: Mutex<Option<BedrockConverseRequest>>,
        response: BedrockConverseResponse,
    }

    impl RecordingTransport {
        fn new(response: BedrockConverseResponse) -> Self {
            Self {
                request: Mutex::new(None),
                response,
            }
        }
    }

    impl BedrockTransport for RecordingTransport {
        fn converse(
            &self,
            _config: &BedrockConfig,
            _model_id: &str,
            request: &BedrockConverseRequest,
        ) -> greentic_dw_llm::LlmResult<BedrockConverseResponse> {
            *self.request.lock().expect("request mutex") = Some(request.clone());
            Ok(self.response.clone())
        }

        fn converse_stream(
            &self,
            _config: &BedrockConfig,
            _model_id: &str,
            request: &BedrockConverseRequest,
        ) -> greentic_dw_llm::LlmResult<BedrockConverseResponse> {
            *self.request.lock().expect("request mutex") = Some(request.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-bedrock").expect("tenant id"),
        )
    }

    fn config() -> BedrockConfig {
        BedrockConfig::new(
            "us-east-1",
            "anthropic.claude-3-haiku-20240307-v1:0",
            10_000,
        )
    }

    fn text_response() -> BedrockConverseResponse {
        BedrockConverseResponse {
            message: BedrockMessage {
                role: BedrockMessageRole::Assistant,
                content: vec![BedrockContentPart::Text("hello from bedrock".to_string())],
            },
            usage: Some(LlmUsage::new(12, 5)),
            stop_reason: Some("end_turn".to_string()),
        }
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = bedrock_config_schema();
        assert_eq!(schema["properties"]["region"]["minLength"], json!(1));
        assert!(schema["properties"]["auth_mode"]["oneOf"].is_array());
    }

    #[test]
    fn config_validation_rejects_empty_model() {
        let mut config = config();
        config.model_id = String::new();
        let err = config.validate().expect_err("invalid config");
        assert_eq!(err.to_string(), "model_id must not be empty");
    }

    #[test]
    fn static_key_auth_requires_values() {
        let mut config = config();
        config.auth_mode = BedrockAuthMode::StaticKeys {
            access_key_id_secret: String::new(),
            secret_access_key_secret: String::new(),
            session_token_secret: None,
        };
        let err = config.validate().expect_err("invalid config");
        assert_eq!(
            err.to_string(),
            "static access key fields must not be empty"
        );
    }

    #[test]
    fn text_request_maps_to_converse_response() {
        let provider =
            BedrockProvider::with_transport(config(), RecordingTransport::new(text_response()))
                .expect("provider");

        let response = provider
            .generate(&tenant(), fixtures::text_request().expect("request"))
            .expect("response");
        assert_eq!(response.text_output(), "hello from bedrock");
        assert_eq!(response.finish_reason, Some(LlmFinishReason::Stop));
    }

    #[test]
    fn tool_request_maps_to_tool_use() {
        let response = BedrockConverseResponse {
            message: BedrockMessage {
                role: BedrockMessageRole::Assistant,
                content: vec![BedrockContentPart::ToolUse {
                    id: "tooluse_1".to_string(),
                    name: "lookup_weather".to_string(),
                    input: json!({ "city": "Rotterdam" }),
                }],
            },
            usage: None,
            stop_reason: Some("tool_use".to_string()),
        };
        let provider = BedrockProvider::with_transport(config(), RecordingTransport::new(response))
            .expect("provider");

        let response = provider
            .generate(&tenant(), fixtures::tool_request().expect("request"))
            .expect("response");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "lookup_weather");
        assert_eq!(response.finish_reason, Some(LlmFinishReason::ToolCalls));
    }

    #[test]
    fn request_payload_carries_any_tool_choice_for_required() {
        let transport = RecordingTransport::new(text_response());
        let provider = BedrockProvider::with_transport(config(), transport).expect("provider");

        let request = fixtures::tool_request()
            .expect("request")
            .with_tool_choice(LlmToolChoice::Required);
        provider.generate(&tenant(), request).expect("response");

        let request = provider.transport.request.lock().expect("request mutex");
        assert_eq!(
            request.as_ref().expect("request").tool_choice,
            LlmToolChoice::Required
        );
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(
            BedrockProvider::<super::AwsSdkBedrockTransport>::provider_id(),
            "dw.llm.bedrock"
        );
        let declaration = BedrockProvider::<super::AwsSdkBedrockTransport>::provider_decl();
        assert_eq!(declaration.provider_type, "dw.llm.bedrock");
    }

    #[test]
    fn wizard_questions_include_aws_auth_fields() {
        let questions = bedrock_wizard_questions();
        assert!(questions.iter().any(|question| question.key == "auth_mode"));
        assert!(
            questions
                .iter()
                .any(|question| question.key == "profile_name")
        );
        assert!(questions.iter().any(|question| {
            question.key == "profile_name"
                && matches!(
                    question.visibility,
                    greentic_dw_providers_common::LlmWizardVisibility::Equals { .. }
                )
        }));
    }

    #[test]
    fn feature_profile_exposes_streaming_and_enterprise_auth() {
        let features = BedrockProvider::<super::AwsSdkBedrockTransport>::default_features();
        assert!(features.chat);
        assert!(features.tool_calling);
        assert!(features.streaming);
        assert!(features.enterprise_auth);
        assert!(!features.local_self_hosted);
    }
}
