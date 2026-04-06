use crate::{
    ObserverVariant, ShortTermMemoryVariant, TaskStoreVariant, observer_capability_uri,
    observer_pack_capability_id, observer_provider_decl, short_term_memory_capability_uri,
    short_term_memory_pack_capability_id, task_store_capability_uri, task_store_pack_capability_id,
};

/// Example deployment channel used by the integration fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndToEndChannel {
    /// Local or OSS-style example using in-memory providers.
    Oss,
    /// Enterprise-style example using Redis-backed providers.
    Enterprise,
}

impl EndToEndChannel {
    /// Returns the channel as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Oss => "oss",
            Self::Enterprise => "enterprise",
        }
    }

    /// Returns the in-repo scenario label used by the docs and fixtures.
    #[must_use]
    pub const fn scenario_name(self) -> &'static str {
        match self {
            Self::Oss => "in-memory",
            Self::Enterprise => "redis",
        }
    }
}

/// Selected provider variants for the end-to-end example.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndToEndVariant {
    /// In-memory providers for local/OSS examples.
    InMemory,
    /// Redis-backed providers for enterprise examples.
    Redis,
}

impl EndToEndVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InMemory => "in-memory",
            Self::Redis => "redis",
        }
    }

    /// Returns the deployment channel associated with this variant.
    #[must_use]
    pub const fn channel(self) -> EndToEndChannel {
        match self {
            Self::InMemory => EndToEndChannel::Oss,
            Self::Redis => EndToEndChannel::Enterprise,
        }
    }

    /// Returns the memory provider variant.
    #[must_use]
    pub const fn memory_variant(self) -> ShortTermMemoryVariant {
        match self {
            Self::InMemory => ShortTermMemoryVariant::InMemory,
            Self::Redis => ShortTermMemoryVariant::Redis,
        }
    }

    /// Returns the state provider variant.
    #[must_use]
    pub const fn state_variant(self) -> TaskStoreVariant {
        match self {
            Self::InMemory => TaskStoreVariant::InMemory,
            Self::Redis => TaskStoreVariant::Redis,
        }
    }

    /// Returns the observer variant used in the example.
    #[must_use]
    pub const fn observer_variant(self) -> ObserverVariant {
        ObserverVariant::BasicAudit
    }
}

/// Operation mapping used in example binding overrides.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationBinding {
    /// Contract operation name.
    pub contract_operation: String,
    /// Component operation name.
    pub component_operation: String,
}

/// Binding override used by the example setup fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingOverride {
    /// Capability request identifier.
    pub request_id: String,
    /// Capability URI being satisfied.
    pub capability: String,
    /// Provider reference used by the setup step.
    pub provider_ref: String,
    /// Selected component reference.
    pub provider_component: String,
    /// Optional operation mapping between contract and component.
    pub operation_map: Vec<OperationBinding>,
}

/// Provider selection used in the example bundle resolution fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleResolutionProvider {
    /// Capability URI resolved by the provider.
    pub capability: String,
    /// Provider type selected for the capability.
    pub provider_type: String,
    /// Pack capability id used by the provider bundle.
    pub pack_capability_id: String,
    /// Component reference used by the provider runtime.
    pub provider_component: String,
}

/// Bundle resolution fixture for the example scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleResolutionFixture {
    /// Bundle identifier.
    pub bundle_id: String,
    /// Human-readable bundle name.
    pub bundle_name: String,
    /// Variant channel used in the example.
    pub variant: EndToEndVariant,
    /// Capability URIs required by the bundle.
    pub required_capabilities: Vec<String>,
    /// Selected providers for the bundle resolution.
    pub providers: Vec<BundleResolutionProvider>,
    /// Final binding overrides applied during setup.
    pub binding_overrides: Vec<BindingOverride>,
    /// Resolved setup files emitted by the example.
    pub generated_setup_files: Vec<String>,
    /// Resolved bundle files emitted by the example.
    pub generated_resolved_files: Vec<String>,
}

/// Bundle metadata used by the example documentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExampleBundleMetadata {
    /// Schema version for the example metadata.
    pub schema_version: u32,
    /// Bundle identifier.
    pub bundle_id: String,
    /// Bundle name.
    pub bundle_name: String,
    /// Requested mode.
    pub requested_mode: String,
    /// Locale used by the example.
    pub locale: String,
    /// Artifact extension emitted by the bundle step.
    pub artifact_extension: String,
    /// Files emitted by the bundle step.
    pub generated_resolved_files: Vec<String>,
    /// Files emitted by the setup step.
    pub generated_setup_files: Vec<String>,
    /// Pack inputs consumed by the example.
    pub app_packs: Vec<String>,
    /// Provider bundles consumed by the example.
    pub extension_providers: Vec<String>,
    /// Capability URIs that the bundle requires.
    pub capabilities: Vec<String>,
    /// Short summary of the example strategy.
    pub notes: Vec<String>,
}

/// Fully assembled example bundle and setup fixture pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndToEndExample {
    /// Metadata for the bundle step.
    pub bundle: ExampleBundleMetadata,
    /// Bundle resolution details.
    pub resolution: BundleResolutionFixture,
}

fn memory_provider_ref(variant: ShortTermMemoryVariant) -> String {
    match variant {
        ShortTermMemoryVariant::InMemory => "component:memory.short-term.in-memory".to_string(),
        ShortTermMemoryVariant::Redis => "component:memory.short-term.redis".to_string(),
    }
}

fn state_provider_ref(variant: TaskStoreVariant) -> String {
    match variant {
        TaskStoreVariant::InMemory => "component:state.task-store.in-memory".to_string(),
        TaskStoreVariant::Redis => "component:state.task-store.redis".to_string(),
    }
}

fn observer_provider_ref() -> String {
    "component:observer.basic-audit".to_string()
}

fn binding_override(
    request_id: impl Into<String>,
    capability: impl Into<String>,
    provider_ref: impl Into<String>,
    provider_component: impl Into<String>,
    operation_map: Vec<OperationBinding>,
) -> BindingOverride {
    BindingOverride {
        request_id: request_id.into(),
        capability: capability.into(),
        provider_ref: provider_ref.into(),
        provider_component: provider_component.into(),
        operation_map,
    }
}

/// Returns the capabilities required by the end-to-end example.
#[must_use]
pub fn end_to_end_required_capabilities() -> Vec<String> {
    vec![
        short_term_memory_capability_uri(),
        task_store_capability_uri(),
        observer_capability_uri("audit"),
    ]
}

/// Returns the bundle metadata for the requested example variant.
#[must_use]
pub fn end_to_end_bundle_metadata(variant: EndToEndVariant) -> ExampleBundleMetadata {
    let channel = variant.channel();
    let provider_suffix = channel.scenario_name();
    let pack_suffix = variant.as_str();

    ExampleBundleMetadata {
        schema_version: 1,
        bundle_id: format!("bundle.dw.providers.{}", channel.as_str()),
        bundle_name: format!("DW Providers {} Example", channel.as_str().to_uppercase()),
        requested_mode: "setup".to_string(),
        locale: "en".to_string(),
        artifact_extension: ".yaml".to_string(),
        generated_resolved_files: vec![format!("resolved/{}.yaml", provider_suffix)],
        generated_setup_files: vec![format!("state/setup/{}.yaml", provider_suffix)],
        app_packs: vec![
            format!("packs/memory-short-term-{}.gtpack", pack_suffix),
            format!("packs/task-store-{}.gtpack", pack_suffix),
            "packs/observer-basic-audit.gtpack".to_string(),
        ],
        extension_providers: vec![
            format!("providers/memory-short-term-{}.gtpack", pack_suffix),
            format!("providers/task-store-{}.gtpack", pack_suffix),
            "providers/observer-basic-audit.gtpack".to_string(),
        ],
        capabilities: end_to_end_required_capabilities(),
        notes: vec![
            "OSS uses in-memory providers and local component refs".to_string(),
            "Enterprise uses redis providers and OCI-style provider refs".to_string(),
        ],
    }
}

/// Returns the setup-time binding overrides for the requested example variant.
#[must_use]
pub fn end_to_end_binding_overrides(variant: EndToEndVariant) -> Vec<BindingOverride> {
    let memory_variant = variant.memory_variant();
    let state_variant = variant.state_variant();
    let memory_component = memory_provider_ref(memory_variant);
    let state_component = state_provider_ref(state_variant);
    let observer_component = observer_provider_ref();

    let memory_provider_ref = match variant {
        EndToEndVariant::InMemory => "component:memory.short-term.in-memory".to_string(),
        EndToEndVariant::Redis => {
            "oci://ghcr.io/greenticai/packs/dw-providers/memory/short-term-redis:latest".to_string()
        }
    };
    let state_provider_ref = match variant {
        EndToEndVariant::InMemory => "component:state.task-store.in-memory".to_string(),
        EndToEndVariant::Redis => {
            "oci://ghcr.io/greenticai/packs/dw-providers/state/task-store-redis:latest".to_string()
        }
    };
    let observer_provider_ref = "component:observer.basic-audit".to_string();

    vec![
        binding_override(
            "require.dw.memory",
            short_term_memory_capability_uri(),
            memory_provider_ref,
            memory_component,
            vec![
                OperationBinding {
                    contract_operation: "get".to_string(),
                    component_operation: "memory.get".to_string(),
                },
                OperationBinding {
                    contract_operation: "put".to_string(),
                    component_operation: "memory.put".to_string(),
                },
            ],
        ),
        binding_override(
            "require.dw.state",
            task_store_capability_uri(),
            state_provider_ref,
            state_component,
            vec![
                OperationBinding {
                    contract_operation: "load".to_string(),
                    component_operation: "state.load".to_string(),
                },
                OperationBinding {
                    contract_operation: "save".to_string(),
                    component_operation: "state.save".to_string(),
                },
            ],
        ),
        binding_override(
            "require.dw.audit",
            observer_capability_uri("audit"),
            observer_provider_ref,
            observer_component,
            vec![OperationBinding {
                contract_operation: "observe".to_string(),
                component_operation: "observer.observe".to_string(),
            }],
        ),
    ]
}

/// Returns the bundle-resolution fixture for the requested example variant.
#[must_use]
pub fn end_to_end_bundle_resolution(variant: EndToEndVariant) -> BundleResolutionFixture {
    let channel = variant.channel();
    let memory_variant = variant.memory_variant();
    let state_variant = variant.state_variant();
    let observer_variant = variant.observer_variant();
    let provider_suffix = channel.scenario_name();

    BundleResolutionFixture {
        bundle_id: format!("bundle.dw.providers.{}", channel.as_str()),
        bundle_name: format!("DW Providers {} Example", channel.as_str().to_uppercase()),
        variant,
        required_capabilities: end_to_end_required_capabilities(),
        providers: vec![
            BundleResolutionProvider {
                capability: short_term_memory_capability_uri(),
                provider_type: crate::provider_type(
                    crate::ProviderCategory::Memory,
                    memory_variant.as_str(),
                ),
                pack_capability_id: short_term_memory_pack_capability_id(),
                provider_component: memory_provider_ref(memory_variant),
            },
            BundleResolutionProvider {
                capability: task_store_capability_uri(),
                provider_type: crate::provider_type(
                    crate::ProviderCategory::State,
                    state_variant.as_str(),
                ),
                pack_capability_id: task_store_pack_capability_id(),
                provider_component: state_provider_ref(state_variant),
            },
            BundleResolutionProvider {
                capability: observer_capability_uri("audit"),
                provider_type: observer_provider_decl(observer_variant).provider_type,
                pack_capability_id: observer_pack_capability_id("audit"),
                provider_component: observer_provider_ref(),
            },
        ],
        binding_overrides: end_to_end_binding_overrides(variant),
        generated_setup_files: vec![format!("state/setup/{}.yaml", provider_suffix)],
        generated_resolved_files: vec![format!("resolved/{}.yaml", provider_suffix)],
    }
}

/// Returns the OSS-style example, which uses in-memory providers.
#[must_use]
pub fn oss_end_to_end_example() -> EndToEndExample {
    let variant = EndToEndVariant::InMemory;
    EndToEndExample {
        bundle: end_to_end_bundle_metadata(variant),
        resolution: end_to_end_bundle_resolution(variant),
    }
}

/// Returns the enterprise-style example, which uses Redis providers.
#[must_use]
pub fn enterprise_end_to_end_example() -> EndToEndExample {
    let variant = EndToEndVariant::Redis;
    EndToEndExample {
        bundle: end_to_end_bundle_metadata(variant),
        resolution: end_to_end_bundle_resolution(variant),
    }
}
