use greentic_dw_providers_common::{
    EndToEndVariant, end_to_end_binding_overrides, end_to_end_bundle_resolution,
    end_to_end_required_capabilities, enterprise_end_to_end_example, oss_end_to_end_example,
};

#[test]
fn oss_example_uses_in_memory_providers() {
    let example = oss_end_to_end_example();

    assert_eq!(example.bundle.bundle_id, "bundle.dw.providers.oss");
    assert_eq!(example.resolution.variant, EndToEndVariant::InMemory);
    assert!(
        example
            .bundle
            .app_packs
            .iter()
            .any(|value| value.contains("packs/dw/memory/short-term-in-memory-pack"))
    );
    assert!(
        example
            .bundle
            .extension_providers
            .iter()
            .any(|value| value.contains("task-store-in-memory"))
    );
}

#[test]
fn enterprise_example_uses_redis_providers() {
    let example = enterprise_end_to_end_example();

    assert_eq!(example.bundle.bundle_id, "bundle.dw.providers.enterprise");
    assert_eq!(example.resolution.variant, EndToEndVariant::Redis);
    assert!(example.resolution.binding_overrides.iter().any(|value| {
        value
            .provider_ref
            .starts_with("oci://ghcr.io/greenticai/packs/dw/")
    }));
    assert!(
        example
            .bundle
            .generated_resolved_files
            .iter()
            .any(|value| value.contains("redis"))
    );
}

#[test]
fn binding_overrides_cover_memory_state_and_audit() {
    let bindings = end_to_end_binding_overrides(EndToEndVariant::InMemory);

    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings[0].request_id, "require.dw.memory");
    assert_eq!(bindings[1].request_id, "require.dw.state");
    assert_eq!(bindings[2].request_id, "require.dw.audit");
    assert!(
        bindings[0]
            .operation_map
            .iter()
            .any(|binding| binding.contract_operation == "get"
                && binding.component_operation == "memory.get")
    );
    assert!(
        bindings[1]
            .operation_map
            .iter()
            .any(|binding| binding.contract_operation == "load"
                && binding.component_operation == "state.load")
    );
}

#[test]
fn bundle_resolution_lists_selected_providers() {
    let resolution = end_to_end_bundle_resolution(EndToEndVariant::Redis);

    assert_eq!(
        resolution.required_capabilities,
        end_to_end_required_capabilities()
    );
    assert_eq!(resolution.providers.len(), 3);
    assert!(
        resolution
            .providers
            .iter()
            .any(|provider| provider.capability == "cap://dw.observer.audit")
    );
    assert!(
        resolution
            .generated_setup_files
            .iter()
            .any(|value| value.contains("redis"))
    );
}
