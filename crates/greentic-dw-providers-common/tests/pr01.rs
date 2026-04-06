use core::str::FromStr;
use greentic_dw_providers_common::{
    ProviderCategory, ProviderDeclSpec, capability_consume, capability_declaration, capability_id,
    capability_offer, capability_profile, capability_provider_ref, capability_requirement,
    capability_uri, pack_capabilities_extension, pack_capability_id, planned_categories,
    provider_decl, provider_manifest, provider_runtime_ref, sample_capability_declaration,
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
    assert_eq!(planned_categories().len(), 6);
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
            ProviderCategory::Memory,
            ProviderCategory::State,
            ProviderCategory::Control,
            ProviderCategory::Observer,
            ProviderCategory::Tool,
        ]
    );
    assert_eq!(workspace_version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(
        workspace_banner(),
        format!(
            "greentic-dw-providers {} scaffold (engine, memory, state, control, observer, tool)",
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
