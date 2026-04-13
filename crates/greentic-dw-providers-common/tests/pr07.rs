use greentic_dw_llm::fixtures;
use greentic_dw_llm::{LlmErrorKind, LlmProviderFeatures, LlmRequest};
use greentic_dw_llm_anthropic::anthropic_config_schema;
use greentic_dw_llm_azure_openai::azure_openai_config_schema;
use greentic_dw_llm_bedrock::bedrock_config_schema;
use greentic_dw_llm_gemini::gemini_config_schema;
use greentic_dw_llm_nvidia_nim::nvidia_nim_config_schema;
use greentic_dw_llm_openai::openai_config_schema;
use greentic_dw_llm_openai_compatible::openai_compatible_config_schema;
use greentic_dw_providers_common::{
    LlmFeatureProfile, LlmFixtureError, LlmWizardProviderQa, PackId, PackManifest, ProviderDecl,
    ProviderManifest, anthropic_llm_feature_profile, anthropic_llm_pack_manifest,
    anthropic_llm_provider_declaration, anthropic_llm_provider_id, anthropic_llm_provider_manifest,
    anthropic_llm_wizard_qa, azure_openai_llm_feature_profile, azure_openai_llm_pack_manifest,
    azure_openai_llm_provider_declaration, azure_openai_llm_provider_id,
    azure_openai_llm_provider_manifest, azure_openai_llm_wizard_qa, bedrock_llm_feature_profile,
    bedrock_llm_pack_manifest, bedrock_llm_provider_declaration, bedrock_llm_provider_id,
    bedrock_llm_provider_manifest, bedrock_llm_wizard_qa, gemini_llm_feature_profile,
    gemini_llm_pack_manifest, gemini_llm_provider_declaration, gemini_llm_provider_id,
    gemini_llm_provider_manifest, gemini_llm_wizard_qa, implemented_llm_wizard_qas,
    llm_pack_capability_id, nvidia_nim_llm_feature_profile, nvidia_nim_llm_pack_manifest,
    nvidia_nim_llm_provider_declaration, nvidia_nim_llm_provider_id,
    nvidia_nim_llm_provider_manifest, nvidia_nim_llm_wizard_qa,
    openai_compatible_llm_feature_profile, openai_compatible_llm_pack_manifest,
    openai_compatible_llm_provider_declaration, openai_compatible_llm_provider_id,
    openai_compatible_llm_provider_manifest, openai_compatible_llm_wizard_qa,
    openai_llm_feature_profile, openai_llm_pack_manifest, openai_llm_provider_declaration,
    openai_llm_provider_id, openai_llm_provider_manifest, openai_llm_wizard_qa,
};
use serde_json::Value;

struct LlmProviderCase {
    provider_name: &'static str,
    model_key: &'static str,
    feature_profile: fn() -> LlmFeatureProfile,
    provider_id: fn() -> String,
    provider_manifest: fn() -> ProviderManifest,
    provider_declaration: fn() -> ProviderDecl,
    pack_manifest: fn(PackId) -> Result<PackManifest, LlmFixtureError>,
    wizard_qa: fn() -> LlmWizardProviderQa,
    config_schema: fn() -> Value,
}

fn llm_provider_cases() -> [LlmProviderCase; 7] {
    [
        LlmProviderCase {
            provider_name: "openai",
            model_key: "model",
            feature_profile: openai_llm_feature_profile,
            provider_id: openai_llm_provider_id,
            provider_manifest: openai_llm_provider_manifest,
            provider_declaration: openai_llm_provider_declaration,
            pack_manifest: openai_llm_pack_manifest,
            wizard_qa: openai_llm_wizard_qa,
            config_schema: openai_config_schema,
        },
        LlmProviderCase {
            provider_name: "azure-openai",
            model_key: "deployment",
            feature_profile: azure_openai_llm_feature_profile,
            provider_id: azure_openai_llm_provider_id,
            provider_manifest: azure_openai_llm_provider_manifest,
            provider_declaration: azure_openai_llm_provider_declaration,
            pack_manifest: azure_openai_llm_pack_manifest,
            wizard_qa: azure_openai_llm_wizard_qa,
            config_schema: azure_openai_config_schema,
        },
        LlmProviderCase {
            provider_name: "openai-compatible",
            model_key: "model",
            feature_profile: openai_compatible_llm_feature_profile,
            provider_id: openai_compatible_llm_provider_id,
            provider_manifest: openai_compatible_llm_provider_manifest,
            provider_declaration: openai_compatible_llm_provider_declaration,
            pack_manifest: openai_compatible_llm_pack_manifest,
            wizard_qa: openai_compatible_llm_wizard_qa,
            config_schema: openai_compatible_config_schema,
        },
        LlmProviderCase {
            provider_name: "anthropic",
            model_key: "model",
            feature_profile: anthropic_llm_feature_profile,
            provider_id: anthropic_llm_provider_id,
            provider_manifest: anthropic_llm_provider_manifest,
            provider_declaration: anthropic_llm_provider_declaration,
            pack_manifest: anthropic_llm_pack_manifest,
            wizard_qa: anthropic_llm_wizard_qa,
            config_schema: anthropic_config_schema,
        },
        LlmProviderCase {
            provider_name: "gemini",
            model_key: "model",
            feature_profile: gemini_llm_feature_profile,
            provider_id: gemini_llm_provider_id,
            provider_manifest: gemini_llm_provider_manifest,
            provider_declaration: gemini_llm_provider_declaration,
            pack_manifest: gemini_llm_pack_manifest,
            wizard_qa: gemini_llm_wizard_qa,
            config_schema: gemini_config_schema,
        },
        LlmProviderCase {
            provider_name: "bedrock",
            model_key: "model_id",
            feature_profile: bedrock_llm_feature_profile,
            provider_id: bedrock_llm_provider_id,
            provider_manifest: bedrock_llm_provider_manifest,
            provider_declaration: bedrock_llm_provider_declaration,
            pack_manifest: bedrock_llm_pack_manifest,
            wizard_qa: bedrock_llm_wizard_qa,
            config_schema: bedrock_config_schema,
        },
        LlmProviderCase {
            provider_name: "nvidia-nim",
            model_key: "model",
            feature_profile: nvidia_nim_llm_feature_profile,
            provider_id: nvidia_nim_llm_provider_id,
            provider_manifest: nvidia_nim_llm_provider_manifest,
            provider_declaration: nvidia_nim_llm_provider_declaration,
            pack_manifest: nvidia_nim_llm_pack_manifest,
            wizard_qa: nvidia_nim_llm_wizard_qa,
            config_schema: nvidia_nim_config_schema,
        },
    ]
}

fn pack_id(provider_name: &str) -> PackId {
    PackId::new(format!("greentic.dw.providers.llm.{provider_name}")).expect("valid pack id")
}

fn required_keys(schema: &Value) -> Vec<&str> {
    schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .map(|item| item.as_str().expect("required entry string"))
        .collect()
}

fn properties(schema: &Value) -> &serde_json::Map<String, Value> {
    schema["properties"].as_object().expect("properties object")
}

fn assert_request_support(
    features: &LlmProviderFeatures,
    request: &LlmRequest,
    supported: bool,
    expected_feature: &str,
) {
    match (supported, features.validate_request(request)) {
        (true, Ok(())) => {}
        (false, Err(err)) => {
            assert_eq!(err.kind, LlmErrorKind::UnsupportedFeature);
            assert_eq!(err.feature.as_deref(), Some(expected_feature));
        }
        (true, Err(err)) => panic!("expected support for {expected_feature}, got {err}"),
        (false, Ok(())) => panic!("expected unsupported feature error for {expected_feature}"),
    }
}

#[test]
fn llm_conformance_provider_metadata_stays_aligned() {
    for case in llm_provider_cases() {
        let features = (case.feature_profile)();
        let provider_id = (case.provider_id)();
        let manifest = (case.provider_manifest)();
        let declaration = (case.provider_declaration)();

        assert_eq!(provider_id, format!("dw.llm.{}", case.provider_name));
        assert_eq!(manifest.provider_type, provider_id);
        assert_eq!(declaration.provider_type, provider_id);
        assert_eq!(
            manifest.capabilities,
            vec![llm_pack_capability_id().to_string()]
        );
        assert_eq!(
            declaration.capabilities,
            vec![llm_pack_capability_id().to_string()]
        );
        assert_eq!(manifest.ops, features.operations());
        assert_eq!(declaration.ops, features.operations());
        assert_eq!(
            declaration.runtime.component_ref,
            format!("component:llm.{}", case.provider_name)
        );
        assert_eq!(
            manifest.config_schema_ref.as_deref(),
            Some(format!("schemas/llm/{}.json", case.provider_name).as_str())
        );
        assert_eq!(
            declaration.docs_ref.as_deref(),
            Some(format!("docs/providers/llm/{}.md", case.provider_name).as_str())
        );
    }
}

#[test]
fn llm_conformance_pack_manifests_embed_shared_family_capability() {
    for case in llm_provider_cases() {
        let manifest = (case.pack_manifest)(pack_id(case.provider_name)).expect("pack manifest");
        let provider_id = (case.provider_id)();
        let provider_extension = manifest
            .provider_extension_inline()
            .expect("provider extension should be present");
        let capabilities_extension = manifest
            .get_capabilities_extension_v1()
            .expect("capabilities extension should decode")
            .expect("capabilities extension should exist");

        assert_eq!(manifest.capabilities.len(), 1);
        assert_eq!(manifest.capabilities[0].name, llm_pack_capability_id());
        assert_eq!(provider_extension.providers.len(), 1);
        assert_eq!(provider_extension.providers[0].provider_type, provider_id);
        assert_eq!(capabilities_extension.offers.len(), 1);
        assert_eq!(
            capabilities_extension.offers[0].cap_id,
            llm_pack_capability_id()
        );
        assert_eq!(
            capabilities_extension.offers[0].provider.component_ref,
            format!("component:llm.{}", case.provider_name)
        );
    }
}

#[test]
fn llm_conformance_feature_profiles_match_core_request_matrix() {
    let text_request = fixtures::text_request().expect("text request fixture");
    let structured_request = fixtures::structured_output_request().expect("structured fixture");
    let tool_request = fixtures::tool_request().expect("tool fixture");
    let stateful_request = fixtures::stateful_request().expect("stateful fixture");
    let multimodal_request = fixtures::multimodal_request().expect("multimodal fixture");

    for case in llm_provider_cases() {
        let features = (case.feature_profile)();

        assert!(
            features.validate_request(&text_request).is_ok(),
            "{} should always support plain text generation",
            case.provider_name
        );
        assert_request_support(
            &features,
            &structured_request,
            features.structured_outputs,
            "structured_outputs",
        );
        assert_request_support(
            &features,
            &tool_request,
            features.tool_calling,
            "tool_calling",
        );
        assert_request_support(
            &features,
            &stateful_request,
            features.stateful_conversation,
            "stateful_conversation",
        );
        assert_request_support(
            &features,
            &multimodal_request,
            features.multimodal_input,
            "multimodal_input",
        );
    }
}

#[test]
fn llm_conformance_config_schemas_expose_family_basics() {
    for case in llm_provider_cases() {
        let features = (case.feature_profile)();
        let schema = (case.config_schema)();
        let required = required_keys(&schema);
        let properties = properties(&schema);

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], false);
        assert!(required.contains(&case.model_key));
        assert!(required.contains(&"timeout_ms"));
        assert_eq!(properties[case.model_key]["type"], "string");
        assert_eq!(properties["timeout_ms"]["type"], "integer");
        assert_eq!(properties["timeout_ms"]["minimum"], 1);

        if features.tool_calling {
            assert_eq!(properties["allow_tools"]["type"], "boolean");
        }

        if features.structured_outputs {
            assert_eq!(properties["allow_structured_outputs"]["type"], "boolean");
        }
    }
}

#[test]
fn llm_conformance_wizard_registry_matches_provider_cases() {
    let registry = implemented_llm_wizard_qas();
    let registry_names = registry
        .iter()
        .map(|provider| provider.provider_name.as_str())
        .collect::<Vec<_>>();
    let case_names = llm_provider_cases()
        .into_iter()
        .map(|case| case.provider_name)
        .collect::<Vec<_>>();

    assert_eq!(registry_names, case_names);

    for (qa, case) in registry.iter().zip(llm_provider_cases()) {
        let schema = (case.config_schema)();
        let properties = properties(&schema);

        assert_eq!(qa, &(case.wizard_qa)());
        assert_eq!(qa.provider_name, case.provider_name);
        assert!(!qa.provider_label_key.is_empty());
        assert!(qa.questions.iter().any(
            |question| question.key == case.model_key || properties.contains_key(&question.key)
        ));
    }
}
