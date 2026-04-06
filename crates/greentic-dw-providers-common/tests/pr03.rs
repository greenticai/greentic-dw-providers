use greentic_dw_providers_common::{
    TaskStoreVariant, task_store_backend_kind, task_store_capability_id, task_store_capability_uri,
    task_store_checkpoint_key, task_store_checkpoint_path, task_store_operations,
    task_store_pack_manifest, task_store_pack_manifest_cbor, task_store_pending_await_path,
    task_store_provider_decl, task_store_provider_extension_inline, task_store_resume_lookup_key,
};
use greentic_types::{PackId, StateBackendKind, decode_pack_manifest, encode_pack_manifest};

fn pack_id() -> PackId {
    match PackId::new("vendor.state.task-store") {
        Ok(value) => value,
        Err(err) => panic!("pack id should be valid: {err}"),
    }
}

#[test]
fn task_store_contract_is_shared_across_variants() {
    assert_eq!(task_store_capability_uri(), "cap://dw.state.task-store");
    assert_eq!(
        task_store_operations(),
        ["state.load", "state.save", "state.list"]
    );
    assert_eq!(
        task_store_provider_decl(TaskStoreVariant::InMemory).provider_type,
        "dw.state.task-store.in-memory"
    );
    assert_eq!(
        task_store_provider_decl(TaskStoreVariant::Redis).provider_type,
        "dw.state.task-store.redis"
    );
}

#[test]
fn task_store_backend_kind_tracks_variant() {
    let memory = task_store_backend_kind(TaskStoreVariant::InMemory, None);
    assert!(matches!(
        memory,
        StateBackendKind::Memory {
            max_entries: 0,
            default_ttl_seconds: 0,
        }
    ));

    let redis = task_store_backend_kind(
        TaskStoreVariant::Redis,
        Some("redis://cache.example/1".to_string()),
    );
    assert!(matches!(
        redis,
        StateBackendKind::Redis {
            redis_url,
            key_prefix,
            default_ttl_seconds: 0,
            pool_size: 5,
            tls_enabled: false,
        } if redis_url == "redis://cache.example/1" && key_prefix == "greentic"
    ));
}

#[test]
fn task_store_keys_and_paths_are_stable() {
    assert_eq!(
        task_store_checkpoint_key("task-42").as_str(),
        "dw:state:checkpoint:task-42"
    );
    assert_eq!(
        task_store_resume_lookup_key("task-42").as_str(),
        "dw:state:resume:task-42"
    );
    assert_eq!(
        task_store_checkpoint_path("task-42").to_pointer(),
        "/state/task-store/checkpoint/task-42"
    );
    assert_eq!(
        task_store_pending_await_path("task-42").to_pointer(),
        "/state/task-store/pending-await/task-42"
    );
}

#[test]
fn task_store_provider_extension_contains_both_variants() {
    let extension = task_store_provider_extension_inline(vec![
        TaskStoreVariant::InMemory,
        TaskStoreVariant::Redis,
    ]);

    assert_eq!(extension.providers.len(), 2);
    assert_eq!(
        extension.providers[0].provider_type,
        "dw.state.task-store.in-memory"
    );
    assert_eq!(
        extension.providers[1].provider_type,
        "dw.state.task-store.redis"
    );
}

#[test]
fn task_store_pack_manifest_roundtrips_to_cbor() {
    let manifest = match task_store_pack_manifest(pack_id(), TaskStoreVariant::InMemory) {
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
fn task_store_pack_manifest_cbor_roundtrips() {
    let bytes = match task_store_pack_manifest_cbor(pack_id(), TaskStoreVariant::Redis) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest CBOR should build: {err}"),
    };

    let decoded = match decode_pack_manifest(&bytes) {
        Ok(value) => value,
        Err(err) => panic!("pack manifest should decode: {err}"),
    };

    let capabilities = decoded
        .get_capabilities_extension_v1()
        .expect("capabilities extension should decode")
        .expect("capabilities extension should be present");
    assert_eq!(
        capabilities.offers[0].cap_id,
        "greentic.cap.state.task-store"
    );
}

#[test]
fn task_store_capability_id_is_canonical() {
    let capability_id = match task_store_capability_id() {
        Ok(value) => value,
        Err(err) => panic!("capability id should build: {err}"),
    };
    assert_eq!(capability_id.as_str(), "cap://dw.state.task-store");
}
