use greentic_dw_providers_common::{
    ControlVariant, ObserverVariant, ToolVariant, control_capability_declaration,
    control_capability_id, control_capability_uri, control_operations, control_pack_manifest,
    control_pack_manifest_cbor, control_provider_decl, control_provider_extension_inline,
    control_variant_from_pack_capability_id, observer_capability_declaration,
    observer_capability_id, observer_capability_uri, observer_operations, observer_pack_manifest,
    observer_pack_manifest_cbor, observer_provider_decl, observer_provider_extension_inline,
    observer_variant_from_pack_capability_id, tool_capability_declaration, tool_capability_id,
    tool_capability_uri, tool_operations, tool_pack_manifest, tool_pack_manifest_cbor,
    tool_provider_decl, tool_provider_extension_inline, tool_variant_from_pack_capability_id,
};
use greentic_types::{PackId, decode_pack_manifest, encode_pack_manifest};

fn control_pack_id() -> PackId {
    match PackId::new("vendor.control.sample") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

fn observer_pack_id() -> PackId {
    match PackId::new("vendor.observer.sample") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

fn tool_pack_id() -> PackId {
    match PackId::new("vendor.tool.sample") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

#[test]
fn control_contract_is_shared_across_variants() {
    assert_eq!(control_capability_uri("basic"), "cap://dw.control.basic");
    assert_eq!(
        control_capability_uri("delegation-guard"),
        "cap://dw.control.delegation-guard"
    );
    assert_eq!(control_operations(), ["control.evaluate", "control.guard"]);
    assert_eq!(
        control_provider_decl(ControlVariant::BasicPolicy).provider_type,
        "dw.control.basic-policy"
    );
    assert_eq!(
        control_provider_decl(ControlVariant::DelegationGuard).provider_type,
        "dw.control.delegation-guard"
    );
}

#[test]
fn control_capability_declaration_is_valid_for_each_backend() {
    let basic = match control_capability_declaration(ControlVariant::BasicPolicy) {
        Ok(value) => value,
        Err(err) => panic!("basic declaration should build: {err}"),
    };
    let guard = match control_capability_declaration(ControlVariant::DelegationGuard) {
        Ok(value) => value,
        Err(err) => panic!("delegation guard declaration should build: {err}"),
    };

    assert_eq!(
        basic.offers[0].capability.as_str(),
        "cap://dw.control.basic"
    );
    assert_eq!(
        guard.offers[0].capability.as_str(),
        "cap://dw.control.delegation-guard"
    );
    assert!(greentic_dw_providers_common::validate_capability_declaration(&basic).is_ok());
    assert!(greentic_dw_providers_common::validate_capability_declaration(&guard).is_ok());
}

#[test]
fn control_provider_extension_contains_both_variants() {
    let extension = control_provider_extension_inline(vec![
        ControlVariant::BasicPolicy,
        ControlVariant::DelegationGuard,
    ]);

    assert_eq!(extension.providers.len(), 2);
    assert_eq!(
        extension.providers[0].provider_type,
        "dw.control.basic-policy"
    );
    assert_eq!(
        extension.providers[1].provider_type,
        "dw.control.delegation-guard"
    );
}

#[test]
fn control_pack_manifests_roundtrip_to_cbor() {
    let manifest = match control_pack_manifest(control_pack_id(), ControlVariant::BasicPolicy) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should build: {err}"),
    };

    let bytes = match encode_pack_manifest(&manifest) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should encode: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };
    assert_eq!(decoded.pack_id, control_pack_id());
}

#[test]
fn control_pack_manifest_cbor_roundtrip() {
    let bytes = match control_pack_manifest_cbor(control_pack_id(), ControlVariant::DelegationGuard)
    {
        Ok(value) => value,
        Err(err) => panic!("pack manifest CBOR should build: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };

    assert_eq!(decoded.pack_id, control_pack_id());
    let extension = decoded
        .get_capabilities_extension_v1()
        .expect("capabilities extension should decode")
        .expect("capabilities extension should be present");
    assert_eq!(
        extension.offers[0].cap_id,
        "greentic.cap.control.delegation-guard"
    );
}

#[test]
fn control_capability_id_is_canonical() {
    let capability_id = match control_capability_id("basic") {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };
    assert_eq!(capability_id.as_str(), "cap://dw.control.basic");
}

#[test]
fn control_selection_helper_resolves_pack_capability_ids() {
    assert_eq!(
        control_variant_from_pack_capability_id("greentic.cap.control.basic"),
        Some(ControlVariant::BasicPolicy)
    );
    assert_eq!(
        control_variant_from_pack_capability_id("greentic.cap.control.delegation-guard"),
        Some(ControlVariant::DelegationGuard)
    );
    assert_eq!(control_variant_from_pack_capability_id("unknown"), None);
}

#[test]
fn observer_contract_is_shared_across_variants() {
    assert_eq!(observer_capability_uri("audit"), "cap://dw.observer.audit");
    assert_eq!(
        observer_capability_uri("metrics"),
        "cap://dw.observer.metrics"
    );
    assert_eq!(
        observer_operations(),
        ["observer.observe", "observer.report"]
    );
    assert_eq!(
        observer_provider_decl(ObserverVariant::BasicAudit).provider_type,
        "dw.observer.basic-audit"
    );
    assert_eq!(
        observer_provider_decl(ObserverVariant::BasicMetrics).provider_type,
        "dw.observer.basic-metrics"
    );
}

#[test]
fn observer_capability_declaration_is_valid_for_each_backend() {
    let audit = match observer_capability_declaration(ObserverVariant::BasicAudit) {
        Ok(value) => value,
        Err(err) => panic!("audit declaration should build: {err}"),
    };
    let metrics = match observer_capability_declaration(ObserverVariant::BasicMetrics) {
        Ok(value) => value,
        Err(err) => panic!("metrics declaration should build: {err}"),
    };

    assert_eq!(
        audit.offers[0].capability.as_str(),
        "cap://dw.observer.audit"
    );
    assert_eq!(
        metrics.offers[0].capability.as_str(),
        "cap://dw.observer.metrics"
    );
    assert!(greentic_dw_providers_common::validate_capability_declaration(&audit).is_ok());
    assert!(greentic_dw_providers_common::validate_capability_declaration(&metrics).is_ok());
}

#[test]
fn observer_provider_extension_contains_both_variants() {
    let extension = observer_provider_extension_inline(vec![
        ObserverVariant::BasicAudit,
        ObserverVariant::BasicMetrics,
    ]);

    assert_eq!(extension.providers.len(), 2);
    assert_eq!(
        extension.providers[0].provider_type,
        "dw.observer.basic-audit"
    );
    assert_eq!(
        extension.providers[1].provider_type,
        "dw.observer.basic-metrics"
    );
}

#[test]
fn observer_pack_manifests_roundtrip_to_cbor() {
    let manifest = match observer_pack_manifest(observer_pack_id(), ObserverVariant::BasicAudit) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should build: {err}"),
    };

    let bytes = match encode_pack_manifest(&manifest) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should encode: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };
    assert_eq!(decoded.pack_id, observer_pack_id());
}

#[test]
fn observer_pack_manifest_cbor_roundtrip() {
    let bytes = match observer_pack_manifest_cbor(observer_pack_id(), ObserverVariant::BasicMetrics)
    {
        Ok(value) => value,
        Err(err) => panic!("pack manifest CBOR should build: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };

    assert_eq!(decoded.pack_id, observer_pack_id());
    let extension = decoded
        .get_capabilities_extension_v1()
        .expect("capabilities extension should decode")
        .expect("capabilities extension should be present");
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.observer.metrics");
}

#[test]
fn observer_capability_id_is_canonical() {
    let capability_id = match observer_capability_id("audit") {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };
    assert_eq!(capability_id.as_str(), "cap://dw.observer.audit");
}

#[test]
fn observer_selection_helper_resolves_pack_capability_ids() {
    assert_eq!(
        observer_variant_from_pack_capability_id("greentic.cap.observer.audit"),
        Some(ObserverVariant::BasicAudit)
    );
    assert_eq!(
        observer_variant_from_pack_capability_id("greentic.cap.observer.metrics"),
        Some(ObserverVariant::BasicMetrics)
    );
    assert_eq!(observer_variant_from_pack_capability_id("unknown"), None);
}

#[test]
fn tool_contract_is_shared_across_variants() {
    assert_eq!(tool_capability_uri("component"), "cap://dw.tool.component");
    assert_eq!(tool_capability_uri("mcp"), "cap://dw.tool.mcp");
    assert_eq!(tool_operations(), ["tool.invoke", "tool.describe"]);
    assert_eq!(
        tool_provider_decl(ToolVariant::ComponentAdapter).provider_type,
        "dw.tool.component-adapter"
    );
    assert_eq!(
        tool_provider_decl(ToolVariant::McpAdapter).provider_type,
        "dw.tool.mcp-adapter"
    );
}

#[test]
fn tool_capability_declaration_is_valid_for_each_backend() {
    let component = match tool_capability_declaration(ToolVariant::ComponentAdapter) {
        Ok(value) => value,
        Err(err) => panic!("component declaration should build: {err}"),
    };
    let mcp = match tool_capability_declaration(ToolVariant::McpAdapter) {
        Ok(value) => value,
        Err(err) => panic!("mcp declaration should build: {err}"),
    };

    assert_eq!(
        component.offers[0].capability.as_str(),
        "cap://dw.tool.component"
    );
    assert_eq!(mcp.offers[0].capability.as_str(), "cap://dw.tool.mcp");
    assert!(greentic_dw_providers_common::validate_capability_declaration(&component).is_ok());
    assert!(greentic_dw_providers_common::validate_capability_declaration(&mcp).is_ok());
}

#[test]
fn tool_provider_extension_contains_both_variants() {
    let extension = tool_provider_extension_inline(vec![
        ToolVariant::ComponentAdapter,
        ToolVariant::McpAdapter,
    ]);

    assert_eq!(extension.providers.len(), 2);
    assert_eq!(
        extension.providers[0].provider_type,
        "dw.tool.component-adapter"
    );
    assert_eq!(extension.providers[1].provider_type, "dw.tool.mcp-adapter");
}

#[test]
fn tool_pack_manifests_roundtrip_to_cbor() {
    let manifest = match tool_pack_manifest(tool_pack_id(), ToolVariant::ComponentAdapter) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should build: {err}"),
    };

    let bytes = match encode_pack_manifest(&manifest) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should encode: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };
    assert_eq!(decoded.pack_id, tool_pack_id());
}

#[test]
fn tool_pack_manifest_cbor_roundtrip() {
    let bytes = match tool_pack_manifest_cbor(tool_pack_id(), ToolVariant::McpAdapter) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest CBOR should build: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };

    assert_eq!(decoded.pack_id, tool_pack_id());
    let extension = decoded
        .get_capabilities_extension_v1()
        .expect("capabilities extension should decode")
        .expect("capabilities extension should be present");
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.tool.mcp");
}

#[test]
fn tool_capability_id_is_canonical() {
    let capability_id = match tool_capability_id("component") {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };
    assert_eq!(capability_id.as_str(), "cap://dw.tool.component");
}

#[test]
fn tool_selection_helper_resolves_pack_capability_ids() {
    assert_eq!(
        tool_variant_from_pack_capability_id("greentic.cap.tool.component"),
        Some(ToolVariant::ComponentAdapter)
    );
    assert_eq!(
        tool_variant_from_pack_capability_id("greentic.cap.tool.mcp"),
        Some(ToolVariant::McpAdapter)
    );
    assert_eq!(tool_variant_from_pack_capability_id("unknown"), None);
}
