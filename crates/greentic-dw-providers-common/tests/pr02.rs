use greentic_dw_providers_common::{
    ShortTermMemoryVariant, short_term_memory_capability_declaration,
    short_term_memory_capability_id, short_term_memory_capability_uri,
    short_term_memory_operations, short_term_memory_pack_manifest,
    short_term_memory_pack_manifest_cbor, short_term_memory_provider_decl,
    short_term_memory_provider_extension_inline,
};
use greentic_types::{PackId, decode_pack_manifest, encode_pack_manifest};

fn pack_id() -> PackId {
    match PackId::new("vendor.memory.short-term") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

#[test]
fn short_term_memory_contract_is_shared_across_variants() {
    assert_eq!(
        short_term_memory_capability_uri(),
        "cap://dw.memory.short-term"
    );
    assert_eq!(
        short_term_memory_operations(),
        ["memory.get", "memory.put", "memory.delete", "memory.clear"]
    );
    assert_eq!(
        short_term_memory_provider_decl(ShortTermMemoryVariant::InMemory).provider_type,
        "dw.memory.short-term.in-memory"
    );
    assert_eq!(
        short_term_memory_provider_decl(ShortTermMemoryVariant::Redis).provider_type,
        "dw.memory.short-term.redis"
    );
}

#[test]
fn short_term_memory_capability_declaration_is_valid_for_each_backend() {
    let in_memory = match short_term_memory_capability_declaration(ShortTermMemoryVariant::InMemory)
    {
        Ok(value) => value,
        Err(err) => panic!("in-memory declaration should build: {err}"),
    };
    let redis = match short_term_memory_capability_declaration(ShortTermMemoryVariant::Redis) {
        Ok(value) => value,
        Err(err) => panic!("redis declaration should build: {err}"),
    };

    assert_eq!(in_memory.offers[0].capability, redis.offers[0].capability);
    assert!(greentic_dw_providers_common::validate_capability_declaration(&in_memory).is_ok());
    assert!(greentic_dw_providers_common::validate_capability_declaration(&redis).is_ok());
}

#[test]
fn short_term_memory_pack_manifests_roundtrip_to_cbor() {
    let manifest =
        match short_term_memory_pack_manifest(pack_id(), ShortTermMemoryVariant::InMemory) {
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
fn short_term_memory_pack_manifest_cbor_roundtrip() {
    let bytes = match short_term_memory_pack_manifest_cbor(pack_id(), ShortTermMemoryVariant::Redis)
    {
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
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.memory.short-term");
}

#[test]
fn short_term_memory_provider_extension_contains_both_variants() {
    let extension = short_term_memory_provider_extension_inline(vec![
        ShortTermMemoryVariant::InMemory,
        ShortTermMemoryVariant::Redis,
    ]);

    assert_eq!(extension.providers.len(), 2);
    assert_eq!(
        extension.providers[0].provider_type,
        "dw.memory.short-term.in-memory"
    );
    assert_eq!(
        extension.providers[1].provider_type,
        "dw.memory.short-term.redis"
    );
}

#[test]
fn short_term_memory_capability_id_is_canonical() {
    let capability_id = match short_term_memory_capability_id() {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };

    assert_eq!(capability_id.as_str(), "cap://dw.memory.short-term");
}
