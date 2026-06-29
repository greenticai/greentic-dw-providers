use core::str::FromStr;
use greentic_dw_providers_common::{
    LlmWizardProviderQa, LlmWizardQuestion, LlmWizardQuestionKind, LlmWizardQuestionOption,
    LlmWizardVisibility, ProviderCategory, ProviderDeclSpec, anthropic_llm_feature_profile,
    anthropic_llm_pack_manifest, anthropic_llm_provider_declaration, anthropic_llm_provider_id,
    anthropic_llm_provider_manifest, anthropic_llm_wizard_qa, azure_openai_llm_feature_profile,
    azure_openai_llm_pack_manifest, azure_openai_llm_provider_declaration,
    azure_openai_llm_provider_id, azure_openai_llm_provider_manifest, azure_openai_llm_wizard_qa,
    bedrock_llm_feature_profile, bedrock_llm_pack_manifest, bedrock_llm_provider_declaration,
    bedrock_llm_provider_id, bedrock_llm_provider_manifest, bedrock_llm_wizard_qa,
    capability_consume, capability_declaration, capability_id, capability_offer,
    capability_profile, capability_provider_ref, capability_requirement, capability_uri,
    gemini_llm_feature_profile, gemini_llm_pack_manifest, gemini_llm_provider_declaration,
    gemini_llm_provider_id, gemini_llm_provider_manifest, gemini_llm_wizard_qa,
    implemented_llm_wizard_qas, llm_capability_declaration, llm_capability_id,
    llm_capability_profile, llm_capability_uri, llm_feature_profile, llm_pack_capabilities,
    llm_pack_capability_id, llm_pack_manifest, llm_provider_declaration, llm_provider_id,
    llm_provider_manifest, llm_provider_pack_capability_id, nvidia_nim_llm_feature_profile,
    nvidia_nim_llm_pack_manifest, nvidia_nim_llm_provider_declaration, nvidia_nim_llm_provider_id,
    nvidia_nim_llm_provider_manifest, nvidia_nim_llm_wizard_qa,
    openai_compatible_llm_feature_profile, openai_compatible_llm_pack_manifest,
    openai_compatible_llm_provider_declaration, openai_compatible_llm_provider_id,
    openai_compatible_llm_provider_manifest, openai_compatible_llm_wizard_qa,
    openai_llm_feature_profile, openai_llm_pack_manifest, openai_llm_provider_declaration,
    openai_llm_provider_id, openai_llm_provider_manifest, openai_llm_wizard_qa,
    pack_capabilities_extension, pack_capability_id, planned_categories, provider_decl,
    provider_manifest, provider_runtime_ref, sample_capability_declaration,
    sample_capability_offer_v1, sample_pack_manifest, sample_pack_manifest_cbor, workspace_banner,
    workspace_version,
};
use greentic_types::{PackId, ProviderDecl, ProviderExtensionInline, decode_pack_manifest};

fn pack_id() -> PackId {
    match PackId::new("vendor.provider.sample") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

#[test]
fn category_helpers_expose_expected_names() {
    assert_eq!(ProviderCategory::Memory.as_str(), "memory");
    assert_eq!(
        ProviderCategory::State.provider_type("store"),
        "dw.state.store"
    );
    assert_eq!(
        capability_uri(ProviderCategory::Engine, "planner"),
        "cap://dw.engine.planner"
    );
    assert_eq!(
        pack_capability_id(ProviderCategory::Tool, "invoke"),
        "greentic.cap.tool.invoke"
    );
    assert_eq!(planned_categories().len(), 9);
}

#[test]
fn provider_helpers_build_valid_structures() {
    let manifest = provider_manifest(
        ProviderCategory::Control,
        "policy",
        vec!["cap://dw.control.policy".to_string()],
        vec!["evaluate".to_string()],
        Some("schemas/policy.json".to_string()),
        None,
    );
    assert!(greentic_dw_providers_common::validate_provider_manifest(&manifest).is_ok());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["greentic.cap.observer.audit".to_string()],
        ops: vec!["watch".to_string()],
        config_schema_ref: "schemas/audit.json".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "component:observer.audit".to_string(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some("docs/providers/audit.md".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_ok());
}

#[test]
fn sample_pack_manifest_includes_provider_and_capability_extensions() {
    let manifest = match sample_pack_manifest(
        pack_id(),
        ProviderCategory::Memory,
        "shortterm",
        "shortterm",
    ) {
        Ok(value) => value,
        Err(err) => panic!("sample pack manifest should build: {err}"),
    };

    assert!(manifest.provider_extension_inline().is_some());
    let extension = match manifest.get_capabilities_extension_v1() {
        Ok(Some(value)) => value,
        Ok(None) => panic!("capabilities extension should be present"),
        Err(err) => panic!("capabilities extension should decode: {err}"),
    };
    assert_eq!(extension.offers.len(), 1);
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.memory.shortterm");
}

#[test]
fn sample_pack_manifest_encodes_to_cbor() {
    let bytes =
        match sample_pack_manifest_cbor(pack_id(), ProviderCategory::Tool, "callout", "invoke") {
            Ok(value) => value,
            Err(err) => panic!("sample pack CBOR should encode: {err}"),
        };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };

    assert_eq!(decoded.pack_id, pack_id());
}

#[test]
fn capability_identifier_helper_is_validated() {
    let cap_id = match capability_id(ProviderCategory::Engine, "plan") {
        Ok(value) => value,
        Err(err) => panic!("capability id should be valid: {err}"),
    };
    assert_eq!(cap_id.as_str(), "cap://dw.engine.plan");
}

#[test]
fn sample_provider_extension_helper_is_usable() {
    let extension = greentic_dw_providers_common::sample_provider_extension_inline(
        ProviderCategory::State,
        "store",
        "kv",
    );

    assert_eq!(extension.providers.len(), 1);
    assert_eq!(extension.providers[0].provider_type, "dw.state.store");
}

#[test]
fn category_and_capability_helpers_cover_remaining_variants() {
    assert_eq!(
        ProviderCategory::from_str("tool"),
        Ok(ProviderCategory::Tool)
    );
    assert_eq!(
        ProviderCategory::from_str("not-a-category"),
        Err("unknown provider category")
    );
    assert_eq!(format!("{}", ProviderCategory::Observer), "observer");
    assert_eq!(
        greentic_dw_providers_common::provider_type(ProviderCategory::Observer, "watch"),
        "dw.observer.watch"
    );
    assert_eq!(
        planned_categories(),
        [
            ProviderCategory::Engine,
            ProviderCategory::Llm,
            ProviderCategory::Memory,
            ProviderCategory::State,
            ProviderCategory::Control,
            ProviderCategory::Observer,
            ProviderCategory::Tool,
            ProviderCategory::Embedding,
            ProviderCategory::Knowledge,
        ]
    );
    assert_eq!(workspace_version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(
        workspace_banner(),
        format!(
            "greentic-dw-providers {} scaffold (engine, llm, memory, state, control, observer, tool, embedding, knowledge)",
            env!("CARGO_PKG_VERSION")
        )
    );

    let provider_ref = capability_provider_ref("component:memory", "provide");
    assert_eq!(provider_ref.component_ref, "component:memory");
    assert_eq!(provider_ref.operation, "provide");

    let offer = capability_offer(
        ProviderCategory::Memory,
        "shortterm",
        "offer.memory",
        "component:memory",
        "provide",
    )
    .expect("valid capability offer");
    assert_eq!(offer.id, "offer.memory");
    assert_eq!(offer.capability.as_str(), "cap://dw.memory.shortterm");
    assert!(offer.provider.is_some());

    let requirement =
        capability_requirement(ProviderCategory::Memory, "shortterm", "require.memory")
            .expect("valid capability requirement");
    assert_eq!(requirement.id, "require.memory");
    assert_eq!(requirement.capability.as_str(), "cap://dw.memory.shortterm");

    let consume = capability_consume(ProviderCategory::Memory, "shortterm", "consume.memory")
        .expect("valid capability consume");
    assert_eq!(consume.id, "consume.memory");
    assert_eq!(consume.capability.as_str(), "cap://dw.memory.shortterm");

    let profile = capability_profile("memory-default");
    assert_eq!(profile.id, "memory-default");

    let declaration = capability_declaration(
        vec![offer.clone()],
        vec![requirement.clone()],
        vec![consume.clone()],
        vec![profile.clone()],
    );
    assert_eq!(declaration.offers.len(), 1);
    assert_eq!(declaration.requires.len(), 1);
    assert_eq!(declaration.consumes.len(), 1);
    assert_eq!(declaration.profiles.len(), 1);
    assert!(greentic_dw_providers_common::validate_capability_declaration(&declaration).is_ok());

    let sample = sample_capability_declaration(
        ProviderCategory::Control,
        "policy",
        "offer.control",
        "component:control",
        "evaluate",
    )
    .expect("sample capability declaration");
    assert_eq!(sample.offers.len(), 1);
    assert_eq!(
        sample.offers[0].capability.as_str(),
        "cap://dw.control.policy"
    );

    let provider_ref_v1 = greentic_types::CapabilityProviderRefV1 {
        component_ref: "component:observer".to_string(),
        op: "watch".to_string(),
    };
    assert_eq!(provider_ref_v1.component_ref, "component:observer");
    assert_eq!(provider_ref_v1.op, "watch");

    let offer_v1 = sample_capability_offer_v1(
        ProviderCategory::Observer,
        "audit",
        "offer.observer",
        "component:observer",
        "watch",
    );
    assert_eq!(offer_v1.offer_id, "offer.observer");
    assert_eq!(offer_v1.cap_id, "greentic.cap.observer.audit");
    assert_eq!(offer_v1.provider.component_ref, "component:observer");

    let extension = pack_capabilities_extension(
        ProviderCategory::Tool,
        "invoke",
        "offer.tool",
        "component:tool",
        "call",
    );
    assert_eq!(extension.offers.len(), 1);
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.tool.invoke");
}

#[test]
fn llm_helpers_build_consistent_family_metadata() {
    let features = llm_feature_profile(true, true, true, true, false, true, false, true);
    assert_eq!(llm_provider_id("openai"), "dw.llm.openai");
    assert_eq!(llm_capability_uri(), "cap://dw.llm");
    assert_eq!(
        llm_capability_id()
            .expect("llm capability id should be valid")
            .as_str(),
        "cap://dw.llm"
    );
    assert_eq!(llm_pack_capability_id(), "greentic.cap.llm");
    assert_eq!(
        llm_provider_pack_capability_id("openai"),
        "greentic.cap.llm.openai"
    );

    let manifest = llm_provider_manifest("openai", &features);
    assert_eq!(manifest.provider_type, "dw.llm.openai");
    assert_eq!(manifest.capabilities, vec!["greentic.cap.llm".to_string()]);
    assert!(manifest.ops.iter().any(|op| op == "llm.chat"));
    assert!(manifest.ops.iter().any(|op| op == "llm.stateful"));

    let decl = llm_provider_declaration("openai", &features);
    assert_eq!(decl.provider_type, "dw.llm.openai");
    assert_eq!(decl.runtime.component_ref, "component:llm.openai");

    let profile = llm_capability_profile("openai", &features);
    assert_eq!(profile.id, "llm.openai.features");
    assert!(
        profile
            .description
            .as_deref()
            .expect("llm profile should include a description")
            .contains("structured_outputs")
    );

    let declaration =
        llm_capability_declaration("openai", &features).expect("llm declaration should build");
    assert_eq!(declaration.offers.len(), 1);
    assert_eq!(declaration.profiles.len(), 1);
    assert_eq!(declaration.offers[0].capability.as_str(), "cap://dw.llm");
}

#[test]
fn llm_wizard_metadata_supports_defaults_and_visibility() {
    let qa = LlmWizardProviderQa::new(
        "demo",
        "wizard.llm.provider.demo.label",
        false,
        vec!["compatible".to_string()],
        vec![
            LlmWizardQuestion::select(
                "mode",
                "wizard.llm.demo.mode",
                true,
                vec![
                    LlmWizardQuestionOption::new("auto", "wizard.llm.demo.mode.auto"),
                    LlmWizardQuestionOption::new("manual", "wizard.llm.demo.mode.manual"),
                ],
            )
            .with_default("auto"),
            LlmWizardQuestion::text("token", "wizard.llm.demo.token", false)
                .visible_when_equals("mode", "manual"),
        ],
    );

    assert_eq!(qa.provider_name, "demo");
    assert_eq!(qa.provider_label_key, "wizard.llm.provider.demo.label");
    assert_eq!(qa.compatibility_tags, vec!["compatible".to_string()]);
    assert_eq!(qa.questions[0].default_value.as_deref(), Some("auto"));
    assert!(matches!(
        qa.questions[0].kind,
        LlmWizardQuestionKind::Select { .. }
    ));
    assert!(matches!(
        qa.questions[1].visibility,
        LlmWizardVisibility::Equals { .. }
    ));
}

#[test]
fn openai_llm_helpers_build_native_provider_metadata() {
    let provider_id = openai_llm_provider_id();
    assert_eq!(provider_id, "dw.llm.openai");

    let features = openai_llm_feature_profile();
    assert!(features.chat);
    assert!(features.structured_outputs);
    assert!(features.tool_calling);
    assert!(features.streaming);
    assert!(features.stateful_conversation);
    assert!(!features.multimodal_input);

    let manifest = openai_llm_provider_manifest();
    assert_eq!(manifest.provider_type, "dw.llm.openai");
    assert_eq!(
        manifest.capabilities,
        vec![llm_pack_capability_id().to_string()]
    );

    let declaration = openai_llm_provider_declaration();
    assert_eq!(declaration.provider_type, "dw.llm.openai");
    assert_eq!(declaration.runtime.component_ref, "component:llm.openai");
    assert!(greentic_dw_providers_common::validate_provider_decl(&declaration).is_ok());

    let pack_manifest =
        openai_llm_pack_manifest(PackId::new("greentic.dw.providers.llm.openai").expect("pack id"))
            .expect("pack manifest");
    assert_eq!(
        pack_manifest.pack_id.to_string(),
        "greentic.dw.providers.llm.openai"
    );
}

#[test]
fn implemented_llm_wizard_registry_covers_current_backends() {
    let providers = implemented_llm_wizard_qas();
    let provider_names = providers
        .iter()
        .map(|provider| provider.provider_name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        provider_names,
        vec![
            "openai",
            "azure-openai",
            "openai-compatible",
            "anthropic",
            "gemini",
            "bedrock",
            "nvidia-nim",
        ]
    );
    assert_eq!(providers[0], openai_llm_wizard_qa());
    assert_eq!(providers[1], azure_openai_llm_wizard_qa());
    assert_eq!(providers[2], openai_compatible_llm_wizard_qa());
    assert_eq!(providers[3], anthropic_llm_wizard_qa());
    assert_eq!(providers[4], gemini_llm_wizard_qa());
    assert_eq!(providers[5], bedrock_llm_wizard_qa());
    assert_eq!(providers[6], nvidia_nim_llm_wizard_qa());
}

#[test]
fn azure_openai_llm_helpers_build_provider_metadata() {
    let provider_id = azure_openai_llm_provider_id();
    assert_eq!(provider_id, "dw.llm.azure-openai");

    let features = azure_openai_llm_feature_profile();
    assert!(features.chat);
    assert!(features.structured_outputs);
    assert!(features.tool_calling);
    assert!(features.streaming);
    assert!(features.stateful_conversation);
    assert!(features.enterprise_auth);
    assert!(!features.local_self_hosted);

    let manifest = azure_openai_llm_provider_manifest();
    assert_eq!(manifest.provider_type, provider_id);

    let declaration = azure_openai_llm_provider_declaration();
    assert_eq!(declaration.provider_type, provider_id);
    assert_eq!(
        declaration.runtime.component_ref,
        "component:llm.azure-openai"
    );

    let pack_manifest = azure_openai_llm_pack_manifest(
        PackId::new("greentic.dw.providers.llm.azure-openai").expect("pack id"),
    )
    .expect("pack manifest");
    assert_eq!(
        pack_manifest.pack_id.to_string(),
        "greentic.dw.providers.llm.azure-openai"
    );
}

#[test]
fn anthropic_llm_helpers_build_provider_metadata() {
    let features = anthropic_llm_feature_profile();
    assert!(features.chat);
    assert!(features.tool_calling);
    assert!(features.structured_outputs);
    assert!(features.enterprise_auth);
    assert_eq!(anthropic_llm_provider_id(), "dw.llm.anthropic");

    let manifest = anthropic_llm_provider_manifest();
    assert_eq!(manifest.provider_type, "dw.llm.anthropic");

    let decl = anthropic_llm_provider_declaration();
    assert_eq!(decl.provider_type, "dw.llm.anthropic");

    let manifest = anthropic_llm_pack_manifest(pack_id()).expect("anthropic pack manifest");
    assert_eq!(manifest.pack_id, pack_id());
}

#[test]
fn openai_compatible_llm_helpers_build_provider_metadata() {
    let provider_id = openai_compatible_llm_provider_id();
    assert_eq!(provider_id, "dw.llm.openai-compatible");

    let features = openai_compatible_llm_feature_profile();
    assert!(features.chat);
    assert!(features.tool_calling);
    assert!(features.streaming);
    assert!(features.local_self_hosted);
    assert!(!features.stateful_conversation);

    let manifest = openai_compatible_llm_provider_manifest();
    assert_eq!(manifest.provider_type, "dw.llm.openai-compatible");

    let declaration = openai_compatible_llm_provider_declaration();
    assert_eq!(declaration.provider_type, "dw.llm.openai-compatible");
    assert_eq!(
        declaration.runtime.component_ref,
        "component:llm.openai-compatible"
    );

    let pack_manifest = openai_compatible_llm_pack_manifest(
        PackId::new("greentic.dw.providers.llm.openai-compatible").expect("pack id"),
    )
    .expect("pack manifest");
    assert_eq!(
        pack_manifest.pack_id.to_string(),
        "greentic.dw.providers.llm.openai-compatible"
    );
}

#[test]
fn gemini_llm_helpers_build_provider_metadata() {
    let provider_id = gemini_llm_provider_id();
    assert_eq!(provider_id, "dw.llm.gemini");

    let features = gemini_llm_feature_profile();
    assert!(features.chat);
    assert!(features.structured_outputs);
    assert!(features.tool_calling);
    assert!(!features.stateful_conversation);
    assert!(!features.local_self_hosted);

    let manifest = gemini_llm_provider_manifest();
    assert_eq!(manifest.provider_type, provider_id);

    let declaration = gemini_llm_provider_declaration();
    assert_eq!(declaration.provider_type, provider_id);
    assert_eq!(declaration.ops[0], "llm.generate");

    let pack_manifest = gemini_llm_pack_manifest(
        PackId::new("greentic.dw.providers.llm.gemini").expect("valid gemini pack id"),
    )
    .expect("gemini pack manifest");
    assert_eq!(
        pack_manifest.capabilities[0].name,
        greentic_dw_providers_common::llm_pack_capability_id()
    );
}

#[test]
fn bedrock_llm_helpers_build_provider_metadata() {
    let provider_id = bedrock_llm_provider_id();
    assert_eq!(provider_id, "dw.llm.bedrock");

    let features = bedrock_llm_feature_profile();
    assert!(features.chat);
    assert!(features.tool_calling);
    assert!(features.streaming);
    assert!(features.enterprise_auth);
    assert!(!features.local_self_hosted);

    let manifest = bedrock_llm_provider_manifest();
    assert_eq!(manifest.provider_type, provider_id);

    let declaration = bedrock_llm_provider_declaration();
    assert_eq!(declaration.provider_type, provider_id);
    assert_eq!(declaration.ops[0], "llm.generate");

    let pack_manifest = bedrock_llm_pack_manifest(
        PackId::new("greentic.dw.providers.llm.bedrock").expect("valid bedrock pack id"),
    )
    .expect("bedrock pack manifest");
    assert_eq!(
        pack_manifest.capabilities[0].name,
        greentic_dw_providers_common::llm_pack_capability_id()
    );
}

#[test]
fn nvidia_nim_llm_helpers_build_provider_metadata() {
    let provider_id = nvidia_nim_llm_provider_id();
    assert_eq!(provider_id, "dw.llm.nvidia-nim");

    let features = nvidia_nim_llm_feature_profile();
    assert!(features.chat);
    assert!(features.tool_calling);
    assert!(features.streaming);
    assert!(features.local_self_hosted);
    assert!(!features.stateful_conversation);

    let manifest = nvidia_nim_llm_provider_manifest();
    assert_eq!(manifest.provider_type, "dw.llm.nvidia-nim");

    let declaration = nvidia_nim_llm_provider_declaration();
    assert_eq!(declaration.provider_type, "dw.llm.nvidia-nim");
    assert_eq!(
        declaration.runtime.component_ref,
        "component:llm.nvidia-nim"
    );

    let pack_manifest = nvidia_nim_llm_pack_manifest(
        PackId::new("greentic.dw.providers.llm.nvidia-nim").expect("pack id"),
    )
    .expect("pack manifest");
    assert_eq!(
        pack_manifest.pack_id.to_string(),
        "greentic.dw.providers.llm.nvidia-nim"
    );
}

#[test]
fn llm_pack_helpers_emit_provider_pack_metadata() {
    let features = llm_feature_profile(true, false, false, true, false, false, true, false);
    let extension = llm_pack_capabilities("ollama");
    assert_eq!(extension.offers.len(), 1);
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.llm");
    assert_eq!(
        extension.offers[0].provider.component_ref,
        "component:llm.ollama"
    );

    let manifest =
        llm_pack_manifest(pack_id(), "ollama", &features).expect("llm pack manifest should build");
    assert_eq!(manifest.pack_id, pack_id());
    assert_eq!(manifest.capabilities[0].name, "greentic.cap.llm");
    assert!(manifest.provider_extension_inline().is_some());
}

#[test]
fn capability_and_provider_validation_rejects_bad_inputs() {
    let err = capability_id(ProviderCategory::Engine, "bad capability").unwrap_err();
    assert!(matches!(
        err,
        greentic_cap_types::CapabilityIdError::InvalidCharacter { ch: ' ', .. }
    ));

    let manifest = provider_manifest(
        ProviderCategory::Control,
        "policy",
        vec!["".to_string()],
        vec!["evaluate".to_string()],
        Some("schemas/policy.json".to_string()),
        None,
    );
    assert!(greentic_dw_providers_common::validate_provider_manifest(&manifest).is_err());

    let manifest = provider_manifest(
        ProviderCategory::Control,
        "policy",
        vec!["cap://dw.control.policy".to_string()],
        vec!["".to_string()],
        Some("schemas/policy.json".to_string()),
        None,
    );
    assert!(greentic_dw_providers_common::validate_provider_manifest(&manifest).is_err());

    let manifest = provider_manifest(
        ProviderCategory::Control,
        "policy",
        vec!["cap://dw.control.policy".to_string()],
        vec!["evaluate".to_string()],
        Some("".to_string()),
        None,
    );
    assert!(greentic_dw_providers_common::validate_provider_manifest(&manifest).is_err());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["".to_string()],
        ops: vec!["watch".to_string()],
        config_schema_ref: "schemas/audit.json".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "component:observer.audit".to_string(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some("docs/providers/audit.md".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_err());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["greentic.cap.observer.audit".to_string()],
        ops: vec!["".to_string()],
        config_schema_ref: "schemas/audit.json".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "component:observer.audit".to_string(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some("docs/providers/audit.md".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_err());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["greentic.cap.observer.audit".to_string()],
        ops: vec!["watch".to_string()],
        config_schema_ref: "".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "component:observer.audit".to_string(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some("docs/providers/audit.md".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_err());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["greentic.cap.observer.audit".to_string()],
        ops: vec!["watch".to_string()],
        config_schema_ref: "schemas/audit.json".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "".to_string(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some("docs/providers/audit.md".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_err());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["greentic.cap.observer.audit".to_string()],
        ops: vec!["watch".to_string()],
        config_schema_ref: "schemas/audit.json".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "component:observer.audit".to_string(),
        export: "".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some("docs/providers/audit.md".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_err());

    let decl = provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: "audit".to_string(),
        capabilities: vec!["greentic.cap.observer.audit".to_string()],
        ops: vec!["watch".to_string()],
        config_schema_ref: "schemas/audit.json".to_string(),
        state_schema_ref: Some("schemas/audit-state.json".to_string()),
        component_ref: "component:observer.audit".to_string(),
        export: "greentic_provider".to_string(),
        world: "".to_string(),
        docs_ref: Some("".to_string()),
    });
    assert!(greentic_dw_providers_common::validate_provider_decl(&decl).is_err());

    let bad_extension = ProviderExtensionInline {
        providers: vec![
            ProviderDecl {
                provider_type: "dw.observer.audit".to_string(),
                capabilities: vec!["greentic.cap.observer.audit".to_string()],
                ops: vec!["watch".to_string()],
                config_schema_ref: "schemas/audit.json".to_string(),
                state_schema_ref: Some("schemas/audit-state.json".to_string()),
                runtime: provider_runtime_ref(
                    "component:observer.audit",
                    "greentic_provider",
                    "greentic:provider/runtime",
                ),
                docs_ref: Some("docs/providers/audit.md".to_string()),
            },
            ProviderDecl {
                provider_type: "dw.observer.audit".to_string(),
                capabilities: vec!["greentic.cap.observer.audit".to_string()],
                ops: vec!["watch".to_string()],
                config_schema_ref: "schemas/audit.json".to_string(),
                state_schema_ref: Some("schemas/audit-state.json".to_string()),
                runtime: provider_runtime_ref(
                    "component:observer.audit-2",
                    "greentic_provider",
                    "greentic:provider/runtime",
                ),
                docs_ref: Some("docs/providers/audit.md".to_string()),
            },
        ],
        additional_fields: Default::default(),
    };
    assert!(
        greentic_dw_providers_common::validate_provider_extension_inline(&bad_extension).is_err()
    );
}
