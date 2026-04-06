use greentic_dw_providers_common::{
    ProviderCategory, ProviderDeclSpec, ProviderExtensionInline, provider_decl,
    provider_extension_inline,
};

/// Builds a provider extension with unique provider types for perf tests and benches.
#[must_use]
pub fn provider_extension_fixture(provider_count: usize) -> ProviderExtensionInline {
    let providers = (0..provider_count)
        .map(|index| {
            let provider_name = format!("perf-provider-{index}");
            let capability_name = format!("perf-capability-{index}");
            provider_decl(ProviderDeclSpec {
                category: ProviderCategory::Control,
                provider_name: provider_name.clone(),
                capabilities: vec![format!("greentic.cap.control.{capability_name}")],
                ops: vec!["evaluate".to_string()],
                config_schema_ref: format!("schemas/{provider_name}.json"),
                state_schema_ref: None,
                component_ref: format!("component:control.{provider_name}"),
                export: "greentic_provider".to_string(),
                world: "greentic:provider/runtime".to_string(),
                docs_ref: Some(format!("docs/providers/{provider_name}.md")),
            })
        })
        .collect();

    provider_extension_inline(providers)
}
