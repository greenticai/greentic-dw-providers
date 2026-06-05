use greentic_dw_providers_common::{
    LongTermMemoryVariant, long_term_memory_capability_declaration, long_term_memory_capability_id,
    long_term_memory_capability_uri, long_term_memory_operations, long_term_memory_pack_manifest,
    long_term_memory_pack_manifest_cbor, long_term_memory_provider_decl,
    long_term_memory_provider_extension_inline,
};
use greentic_types::{PackId, decode_pack_manifest, encode_pack_manifest};

fn pack_id() -> PackId {
    match PackId::new("vendor.memory.long-term") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

#[test]
fn long_term_memory_contract_is_shared_across_variants() {
    assert_eq!(
        long_term_memory_capability_uri(),
        "cap://dw.memory.long-term"
    );
    assert_eq!(
        long_term_memory_operations(),
        ["memory.ingest", "memory.recall"]
    );
    assert_eq!(
        long_term_memory_provider_decl(LongTermMemoryVariant::Chronicle).provider_type,
        "dw.memory.long-term.chronicle"
    );
}

#[test]
fn long_term_memory_variant_strings() {
    assert_eq!(LongTermMemoryVariant::Chronicle.as_str(), "chronicle");
    assert_eq!(
        LongTermMemoryVariant::Chronicle.component_ref(),
        "component:memory.chronicle"
    );
    assert_eq!(
        LongTermMemoryVariant::Chronicle.provider_type(),
        "dw.memory.long-term.chronicle"
    );
}

#[test]
fn long_term_memory_pack_capability_id_is_canonical() {
    assert_eq!(
        greentic_dw_providers_common::long_term_memory_pack_capability_id(),
        "greentic.cap.memory.long-term"
    );
}

#[test]
fn long_term_memory_capability_id_is_canonical() {
    let capability_id = match long_term_memory_capability_id() {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };

    assert_eq!(capability_id.as_str(), "cap://dw.memory.long-term");
}

#[test]
fn long_term_memory_capability_declaration_is_valid_for_chronicle() {
    let chronicle = match long_term_memory_capability_declaration(LongTermMemoryVariant::Chronicle)
    {
        Ok(value) => value,
        Err(err) => panic!("chronicle declaration should build: {err}"),
    };

    assert!(greentic_dw_providers_common::validate_capability_declaration(&chronicle).is_ok());
}

#[test]
fn long_term_memory_pack_manifests_roundtrip_to_cbor() {
    let manifest = match long_term_memory_pack_manifest(pack_id(), LongTermMemoryVariant::Chronicle)
    {
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
fn long_term_memory_pack_manifest_cbor_roundtrip() {
    let bytes =
        match long_term_memory_pack_manifest_cbor(pack_id(), LongTermMemoryVariant::Chronicle) {
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
    assert_eq!(extension.offers[0].cap_id, "greentic.cap.memory.long-term");
}

#[test]
fn long_term_memory_provider_extension_contains_chronicle_variant() {
    let extension =
        long_term_memory_provider_extension_inline(vec![LongTermMemoryVariant::Chronicle]);

    assert_eq!(extension.providers.len(), 1);
    assert_eq!(
        extension.providers[0].provider_type,
        "dw.memory.long-term.chronicle"
    );
}
