#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! NVIDIA NIM provider built on top of the shared normalized LLM contract.

mod config;
mod discovery;
mod errors;
mod mapping;
mod qa;
mod transport;

pub use config::{
    NvidiaNimApiMode, NvidiaNimConfig, NvidiaNimFeatureOverride, nvidia_nim_config_schema,
};
pub use discovery::{
    NvidiaNimHealth, NvidiaNimModel, parse_health_response, parse_models_response,
};
pub use errors::NvidiaNimError;
pub use qa::{NvidiaNimWizardQuestion, nvidia_nim_wizard_qa, nvidia_nim_wizard_questions};
pub use transport::{HttpNvidiaNimTransport, NvidiaNimTransport};

use greentic_dw_llm::{LlmProvider, LlmProviderFeatures, LlmRequest, LlmResponse, LlmResult};
use greentic_dw_providers_common::{
    nvidia_nim_llm_feature_profile, nvidia_nim_llm_pack_manifest,
    nvidia_nim_llm_provider_declaration, nvidia_nim_llm_provider_id,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use mapping::{build_request, parse_response};

/// Canonical provider slug for the NVIDIA NIM backend.
pub const NVIDIA_NIM_PROVIDER_NAME: &str = "nvidia-nim";

/// Optional startup probe result derived from the NIM-aware config flags.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NvidiaNimStartupReport {
    /// Discovered models when model discovery was requested.
    pub models: Option<Vec<NvidiaNimModel>>,
    /// Ready endpoint status when health checks were requested.
    pub ready: Option<NvidiaNimHealth>,
    /// Live endpoint status when health checks were requested.
    pub live: Option<NvidiaNimHealth>,
}

/// NVIDIA NIM provider implementation.
pub struct NvidiaNimProvider<T = HttpNvidiaNimTransport> {
    config: NvidiaNimConfig,
    features: LlmProviderFeatures,
    transport: T,
}

impl NvidiaNimProvider<HttpNvidiaNimTransport> {
    /// Creates a provider with the default HTTP transport.
    pub fn new(config: NvidiaNimConfig) -> LlmResult<Self> {
        Self::with_transport(config, HttpNvidiaNimTransport)
    }
}

impl<T> NvidiaNimProvider<T>
where
    T: NvidiaNimTransport,
{
    /// Creates a provider with a custom transport.
    pub fn with_transport(config: NvidiaNimConfig, transport: T) -> LlmResult<Self> {
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
        nvidia_nim_llm_provider_id()
    }

    /// Returns the canonical provider declaration.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        nvidia_nim_llm_provider_declaration()
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.llm.nvidia-nim")?;
        nvidia_nim_llm_pack_manifest(pack_id)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the default feature profile for this backend.
    #[must_use]
    pub fn default_features() -> LlmProviderFeatures {
        nvidia_nim_llm_feature_profile()
    }

    /// Returns the provider configuration.
    #[must_use]
    pub fn config(&self) -> &NvidiaNimConfig {
        &self.config
    }

    /// Lists models through the NIM discovery surface.
    pub fn list_models(&self) -> LlmResult<Vec<NvidiaNimModel>> {
        self.require_nim_aware("model discovery")?;
        let value = self.transport.models(&self.config)?;
        parse_models_response(&value)
    }

    /// Checks the ready health endpoint.
    pub fn health_ready(&self) -> LlmResult<NvidiaNimHealth> {
        self.require_nim_aware("ready health checks")?;
        let value = self.transport.ready(&self.config)?;
        parse_health_response(&value)
    }

    /// Checks the live health endpoint.
    pub fn health_live(&self) -> LlmResult<NvidiaNimHealth> {
        self.require_nim_aware("live health checks")?;
        let value = self.transport.live(&self.config)?;
        parse_health_response(&value)
    }

    /// Runs the optional startup probes requested by the provider configuration.
    pub fn probe_startup(&self) -> LlmResult<NvidiaNimStartupReport> {
        let models = if self.config.discover_models_on_start {
            Some(self.list_models()?)
        } else {
            None
        };
        let (ready, live) = if self.config.healthcheck_on_start {
            (Some(self.health_ready()?), Some(self.health_live()?))
        } else {
            (None, None)
        };
        Ok(NvidiaNimStartupReport {
            models,
            ready,
            live,
        })
    }

    fn require_nim_aware(&self, capability: &'static str) -> LlmResult<()> {
        if self.config.is_nim_aware() {
            return Ok(());
        }
        Err(greentic_dw_llm::LlmError::from(NvidiaNimError::config(
            format!("{capability} require api_mode=nim_aware"),
        )))
    }
}

impl<T> LlmProvider for NvidiaNimProvider<T>
where
    T: NvidiaNimTransport,
{
    fn features(&self) -> &LlmProviderFeatures {
        &self.features
    }

    fn generate(&self, _tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse> {
        self.validate_request(&request)?;
        let payload = build_request(&self.config, &request)?;
        let response = self.transport.infer(&self.config, &payload)?;
        parse_response(&response, &request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        NvidiaNimApiMode, NvidiaNimConfig, NvidiaNimProvider, NvidiaNimStartupReport,
        NvidiaNimTransport, nvidia_nim_config_schema, nvidia_nim_wizard_questions,
    };
    use greentic_dw_llm::{LlmErrorKind, LlmProvider, fixtures};
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::{Value, json};
    use std::sync::Mutex;

    struct RecordingTransport {
        inference: Value,
        models: Value,
        ready: Value,
        live: Value,
        payload: Mutex<Option<Value>>,
        model_calls: Mutex<usize>,
        ready_calls: Mutex<usize>,
        live_calls: Mutex<usize>,
    }

    impl RecordingTransport {
        fn new() -> Self {
            Self {
                inference: json!({
                    "id": "chatcmpl_1",
                    "model": "meta/llama-3.1-8b-instruct",
                    "choices": [{
                        "finish_reason": "stop",
                        "message": { "role": "assistant", "content": "hello from nim" }
                    }],
                    "usage": {
                        "prompt_tokens": 12,
                        "completion_tokens": 5,
                        "total_tokens": 17
                    }
                }),
                models: json!({
                    "data": [
                        { "id": "meta/llama-3.1-8b-instruct", "owned_by": "nvidia" },
                        { "id": "mistralai/mixtral-8x7b-instruct-v0.1" }
                    ]
                }),
                ready: json!({ "status": "ready" }),
                live: json!({ "ok": true }),
                payload: Mutex::new(None),
                model_calls: Mutex::new(0),
                ready_calls: Mutex::new(0),
                live_calls: Mutex::new(0),
            }
        }
    }

    impl NvidiaNimTransport for RecordingTransport {
        fn infer(
            &self,
            _config: &NvidiaNimConfig,
            payload: &Value,
        ) -> greentic_dw_llm::LlmResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.inference.clone())
        }

        fn models(&self, _config: &NvidiaNimConfig) -> greentic_dw_llm::LlmResult<Value> {
            *self.model_calls.lock().expect("model calls mutex") += 1;
            Ok(self.models.clone())
        }

        fn ready(&self, _config: &NvidiaNimConfig) -> greentic_dw_llm::LlmResult<Value> {
            *self.ready_calls.lock().expect("ready calls mutex") += 1;
            Ok(self.ready.clone())
        }

        fn live(&self, _config: &NvidiaNimConfig) -> greentic_dw_llm::LlmResult<Value> {
            *self.live_calls.lock().expect("live calls mutex") += 1;
            Ok(self.live.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-nim").expect("tenant id"),
        )
    }

    fn config() -> NvidiaNimConfig {
        let mut config = NvidiaNimConfig::new(
            "http://localhost:8000/v1",
            "meta/llama-3.1-8b-instruct",
            10_000,
        );
        config.api_mode = NvidiaNimApiMode::NimAware;
        config.allow_structured_outputs = true;
        config
    }

    #[test]
    fn config_schema_fixture_exposes_expected_fields() {
        let schema = nvidia_nim_config_schema();
        assert_eq!(
            schema["properties"]["api_mode"]["enum"][0],
            json!("openai_compatible")
        );
        assert_eq!(
            schema["properties"]["api_mode"]["enum"][1],
            json!("nim_aware")
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
    fn config_validation_rejects_startup_probes_in_compatible_mode() {
        let mut config = config();
        config.api_mode = NvidiaNimApiMode::OpenAiCompatible;
        config.discover_models_on_start = true;
        let err = config.validate().expect_err("invalid config");
        assert_eq!(
            err.to_string(),
            "discover_models_on_start and healthcheck_on_start require api_mode=nim_aware"
        );
    }

    #[test]
    fn inference_maps_text_requests() {
        let provider = NvidiaNimProvider::with_transport(config(), RecordingTransport::new())
            .expect("provider");
        let response = provider
            .generate(&tenant(), fixtures::text_request().expect("request"))
            .expect("response");
        assert_eq!(response.text_output(), "hello from nim");
        assert_eq!(response.usage.expect("usage").total_tokens, 17);
    }

    #[test]
    fn discovery_helpers_parse_model_list() {
        let provider = NvidiaNimProvider::with_transport(config(), RecordingTransport::new())
            .expect("provider");
        let models = provider.list_models().expect("models");
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "meta/llama-3.1-8b-instruct");
    }

    #[test]
    fn health_helpers_parse_ready_and_live() {
        let provider = NvidiaNimProvider::with_transport(config(), RecordingTransport::new())
            .expect("provider");
        assert!(provider.health_ready().expect("ready").ok);
        assert!(provider.health_live().expect("live").ok);
    }

    #[test]
    fn provider_metadata_helpers_return_expected_values() {
        assert_eq!(
            NvidiaNimProvider::<super::HttpNvidiaNimTransport>::provider_id(),
            "dw.llm.nvidia-nim"
        );
        let decl = NvidiaNimProvider::<super::HttpNvidiaNimTransport>::provider_decl();
        assert_eq!(decl.provider_type, "dw.llm.nvidia-nim");
        let manifest = NvidiaNimProvider::<super::HttpNvidiaNimTransport>::pack_manifest()
            .expect("pack manifest");
        assert_eq!(
            manifest.pack_id.to_string(),
            "greentic.dw.providers.llm.nvidia-nim"
        );
    }

    #[test]
    fn feature_profile_marks_self_hosted_non_stateful_backend() {
        let provider = NvidiaNimProvider::with_transport(config(), RecordingTransport::new())
            .expect("provider");
        assert!(provider.features().chat);
        assert!(provider.features().streaming);
        assert!(provider.features().local_self_hosted);
        assert!(!provider.features().stateful_conversation);
    }

    #[test]
    fn wizard_questions_include_nim_specific_fields() {
        let questions = nvidia_nim_wizard_questions();
        assert!(questions.iter().any(|question| question.key == "api_mode"));
        assert!(
            questions
                .iter()
                .any(|question| question.key == "discover_models_on_start")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "api_key_secret")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "allow_tools")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "allow_structured_outputs")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "allow_streaming")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.key == "timeout_ms")
        );
        assert!(questions.iter().any(|question| {
            question.key == "discover_models_on_start"
                && matches!(
                    question.visibility,
                    greentic_dw_providers_common::LlmWizardVisibility::Equals { .. }
                )
        }));
    }

    #[test]
    fn startup_probe_honors_discovery_and_health_flags() {
        let mut config = config();
        config.discover_models_on_start = true;
        config.healthcheck_on_start = true;
        let transport = RecordingTransport::new();
        let provider =
            NvidiaNimProvider::with_transport(config, transport).expect("provider should build");

        let report = provider.probe_startup().expect("startup probe");
        assert_eq!(
            report,
            NvidiaNimStartupReport {
                models: Some(vec![
                    super::NvidiaNimModel {
                        id: "meta/llama-3.1-8b-instruct".to_string(),
                        owned_by: Some("nvidia".to_string()),
                    },
                    super::NvidiaNimModel {
                        id: "mistralai/mixtral-8x7b-instruct-v0.1".to_string(),
                        owned_by: None,
                    },
                ]),
                ready: Some(super::NvidiaNimHealth {
                    ok: true,
                    status: Some("ready".to_string()),
                }),
                live: Some(super::NvidiaNimHealth {
                    ok: true,
                    status: None,
                }),
            }
        );
    }

    #[test]
    fn startup_probe_skips_optional_endpoints_when_flags_are_disabled() {
        let transport = RecordingTransport::new();
        let provider =
            NvidiaNimProvider::with_transport(config(), transport).expect("provider should build");

        let report = provider.probe_startup().expect("startup probe");
        assert_eq!(report, NvidiaNimStartupReport::default());
    }

    #[test]
    fn discovery_and_health_helpers_require_nim_aware_mode() {
        let mut config = config();
        config.api_mode = NvidiaNimApiMode::OpenAiCompatible;
        let provider =
            NvidiaNimProvider::with_transport(config, RecordingTransport::new()).expect("provider");

        let discovery = provider
            .list_models()
            .expect_err("discovery should be rejected");
        assert_eq!(discovery.kind, LlmErrorKind::InvalidRequest);
        assert!(
            discovery
                .message
                .contains("model discovery require api_mode=nim_aware")
        );

        let ready = provider
            .health_ready()
            .expect_err("ready health should be rejected");
        assert_eq!(ready.kind, LlmErrorKind::InvalidRequest);
        assert!(
            ready
                .message
                .contains("ready health checks require api_mode=nim_aware")
        );

        let live = provider
            .health_live()
            .expect_err("live health should be rejected");
        assert_eq!(live.kind, LlmErrorKind::InvalidRequest);
        assert!(
            live.message
                .contains("live health checks require api_mode=nim_aware")
        );
    }

    #[test]
    fn discovery_parsing_errors_are_nim_specific() {
        let err = super::parse_models_response(&json!({}))
            .expect_err("discovery parsing should fail for invalid payload");
        assert_eq!(err.kind, LlmErrorKind::Provider);
        assert!(err.message.contains("NIM discovery request failed"));
    }

    #[test]
    fn health_parsing_errors_are_nim_specific() {
        let err = super::parse_health_response(&json!({}))
            .expect_err("health parsing should fail for invalid payload");
        assert_eq!(err.kind, LlmErrorKind::Provider);
        assert!(err.message.contains("NIM health request failed"));
    }

    #[test]
    fn http_error_mapping_distinguishes_inference_and_discovery_paths() {
        let inference = super::mapping::map_http_error(
            404,
            r#"{"error":{"message":"model missing"}}"#,
            "inference",
        );
        assert_eq!(inference.kind, LlmErrorKind::UnsupportedModel);
        assert!(inference.message.contains("NIM inference request failed"));

        let discovery = super::mapping::map_http_error(503, "unavailable", "discovery");
        assert_eq!(discovery.kind, LlmErrorKind::Provider);
        assert!(discovery.retryable);
        assert!(discovery.message.contains("NIM discovery request failed"));
    }
}
