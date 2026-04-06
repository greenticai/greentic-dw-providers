use greentic_dw_providers_common::{
    EngineVariant, engine_capability_declaration, engine_capability_id, engine_capability_uri,
    engine_operations, engine_pack_manifest, engine_pack_manifest_cbor, engine_provider_decl,
    engine_provider_extension_inline, engine_variant_from_pack_capability_id,
};
use greentic_types::{PackId, decode_pack_manifest, encode_pack_manifest};

fn pack_id() -> PackId {
    match PackId::new("vendor.engine.sample") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

#[test]
fn engine_contract_is_shared_across_variants() {
    assert_eq!(engine_capability_uri("default"), "cap://dw.engine.default");
    assert_eq!(engine_capability_uri("router"), "cap://dw.engine.router");
    assert_eq!(engine_operations(), ["engine.decide", "engine.route"]);
    assert_eq!(
        engine_provider_decl(EngineVariant::Default).provider_type,
        "dw.engine.default"
    );
    assert_eq!(
        engine_provider_decl(EngineVariant::RouterLite).provider_type,
        "dw.engine.router-lite"
    );
}

#[test]
fn engine_capability_declaration_is_valid_for_each_backend() {
    let default = match engine_capability_declaration(EngineVariant::Default) {
        Ok(value) => value,
        Err(err) => panic!("default declaration should build: {err}"),
    };
    let router = match engine_capability_declaration(EngineVariant::RouterLite) {
        Ok(value) => value,
        Err(err) => panic!("router declaration should build: {err}"),
    };

    assert_eq!(
        default.offers[0].capability.as_str(),
        "cap://dw.engine.default"
    );
    assert_eq!(
        router.offers[0].capability.as_str(),
        "cap://dw.engine.router"
    );
    assert!(greentic_dw_providers_common::validate_capability_declaration(&default).is_ok());
    assert!(greentic_dw_providers_common::validate_capability_declaration(&router).is_ok());
}

#[test]
fn engine_provider_extension_contains_both_variants() {
    let extension =
        engine_provider_extension_inline(vec![EngineVariant::Default, EngineVariant::RouterLite]);

    assert_eq!(extension.providers.len(), 2);
    assert_eq!(extension.providers[0].provider_type, "dw.engine.default");
    assert_eq!(
        extension.providers[1].provider_type,
        "dw.engine.router-lite"
    );
}

#[test]
fn engine_pack_manifests_roundtrip_to_cbor() {
    let manifest = match engine_pack_manifest(pack_id(), EngineVariant::Default) {
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
    assert_eq!(decoded.pack_id, pack_id());
}

#[test]
fn engine_pack_manifest_cbor_roundtrip() {
    let bytes = match engine_pack_manifest_cbor(pack_id(), EngineVariant::RouterLite) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest CBOR should build: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };

    assert_eq!(decoded.pack_id, pack_id());
    let extension = decoded
        .get_capabilities_extension_v1()
        .expect("capabilities extension should decode")
        .expect("capabilities extension should be present");
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.engine.router");
}

#[test]
fn engine_capability_id_is_canonical() {
    let capability_id = match engine_capability_id("default") {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };
    assert_eq!(capability_id.as_str(), "cap://dw.engine.default");
}

#[test]
fn engine_selection_helper_resolves_pack_capability_ids() {
    assert_eq!(
        engine_variant_from_pack_capability_id("greentic.cap.engine.default"),
        Some(EngineVariant::Default)
    );
    assert_eq!(
        engine_variant_from_pack_capability_id("greentic.cap.engine.router"),
        Some(EngineVariant::RouterLite)
    );
    assert_eq!(engine_variant_from_pack_capability_id("unknown"), None);
}
