#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared support crate for Greentic DW provider scaffolding.
//!
//! The crate keeps the shared provider surface small and reusable:
//! it reexports the canonical Greentic models, adds DW-oriented naming helpers,
//! and provides sample manifest builders that future provider crates can reuse
//! during local development and integration tests.

mod capability;
mod category;
mod control;
mod engine;
mod fixtures;
mod integration;
mod llm;
mod memory;
mod observer;
mod provider;
mod state;
mod tool;

pub use capability::{
    capability_consume, capability_declaration, capability_id, capability_offer,
    capability_profile, capability_provider_ref, capability_requirement, capability_uri,
    pack_capabilities_extension, pack_capability_id, sample_capability_declaration,
    sample_capability_offer_v1, validate_capability_declaration,
};
pub use category::{
    ProviderCategory, planned_categories, provider_type, workspace_banner, workspace_version,
};
pub use control::{
    ControlFixtureError, ControlVariant, control_capability_declaration, control_capability_id,
    control_capability_uri, control_operations, control_pack_capabilities,
    control_pack_capability_id, control_pack_manifest, control_pack_manifest_cbor,
    control_provider_decl, control_provider_extension_inline,
    control_variant_from_pack_capability_id,
};
pub use engine::{
    EngineFixtureError, EngineVariant, engine_capability_declaration, engine_capability_id,
    engine_capability_uri, engine_operations, engine_pack_capabilities, engine_pack_capability_id,
    engine_pack_manifest, engine_pack_manifest_cbor, engine_provider_decl,
    engine_provider_extension_inline, engine_variant_from_pack_capability_id,
};
pub use fixtures::{
    sample_capability_declaration_fixture, sample_pack_manifest, sample_pack_manifest_cbor,
    sample_provider_decl, sample_provider_extension_inline,
};
pub use integration::{
    BindingOverride, BundleResolutionFixture, BundleResolutionProvider, EndToEndChannel,
    EndToEndExample, EndToEndVariant, ExampleBundleMetadata, OperationBinding,
    end_to_end_binding_overrides, end_to_end_bundle_resolution, end_to_end_required_capabilities,
    enterprise_end_to_end_example, oss_end_to_end_example,
};
pub use llm::{
    ANTHROPIC_LLM_PROVIDER_NAME, AZURE_OPENAI_LLM_PROVIDER_NAME, BEDROCK_LLM_PROVIDER_NAME,
    GEMINI_LLM_PROVIDER_NAME, LLM_CAPABILITY_URI, LLM_PACK_CAPABILITY_ID, LlmFeatureProfile,
    LlmFixtureError, LlmWizardProviderQa, LlmWizardQuestion, LlmWizardQuestionKind,
    LlmWizardQuestionOption, LlmWizardVisibility, NVIDIA_NIM_LLM_PROVIDER_NAME,
    OPENAI_COMPATIBLE_LLM_PROVIDER_NAME, OPENAI_LLM_PROVIDER_NAME, anthropic_llm_feature_profile,
    anthropic_llm_pack_manifest, anthropic_llm_provider_declaration, anthropic_llm_provider_id,
    anthropic_llm_provider_manifest, anthropic_llm_wizard_qa, azure_openai_llm_feature_profile,
    azure_openai_llm_pack_manifest, azure_openai_llm_provider_declaration,
    azure_openai_llm_provider_id, azure_openai_llm_provider_manifest, azure_openai_llm_wizard_qa,
    bedrock_llm_feature_profile, bedrock_llm_pack_manifest, bedrock_llm_provider_declaration,
    bedrock_llm_provider_id, bedrock_llm_provider_manifest, bedrock_llm_wizard_qa,
    gemini_llm_feature_profile, gemini_llm_pack_manifest, gemini_llm_provider_declaration,
    gemini_llm_provider_id, gemini_llm_provider_manifest, gemini_llm_wizard_qa,
    implemented_llm_wizard_qas, llm_capability_declaration, llm_capability_id,
    llm_capability_profile, llm_capability_uri, llm_feature_profile, llm_operations,
    llm_pack_capabilities, llm_pack_capability_id, llm_pack_manifest, llm_pack_manifest_cbor,
    llm_provider_declaration, llm_provider_extension_inline, llm_provider_id,
    llm_provider_manifest, llm_provider_pack_capability_id, nvidia_nim_llm_feature_profile,
    nvidia_nim_llm_pack_manifest, nvidia_nim_llm_provider_declaration, nvidia_nim_llm_provider_id,
    nvidia_nim_llm_provider_manifest, nvidia_nim_llm_wizard_qa,
    openai_compatible_llm_feature_profile, openai_compatible_llm_pack_manifest,
    openai_compatible_llm_provider_declaration, openai_compatible_llm_provider_id,
    openai_compatible_llm_provider_manifest, openai_compatible_llm_wizard_qa,
    openai_llm_feature_profile, openai_llm_pack_manifest, openai_llm_provider_declaration,
    openai_llm_provider_id, openai_llm_provider_manifest, openai_llm_wizard_qa,
};
pub use memory::{
    MemoryFixtureError, ShortTermMemoryVariant, short_term_memory_capability_declaration,
    short_term_memory_capability_id, short_term_memory_capability_uri,
    short_term_memory_operations, short_term_memory_pack_capabilities,
    short_term_memory_pack_capability_id, short_term_memory_pack_manifest,
    short_term_memory_pack_manifest_cbor, short_term_memory_provider_decl,
    short_term_memory_provider_extension_inline,
};
pub use observer::{
    ObserverFixtureError, ObserverVariant, observer_capability_declaration, observer_capability_id,
    observer_capability_uri, observer_operations, observer_pack_capabilities,
    observer_pack_capability_id, observer_pack_manifest, observer_pack_manifest_cbor,
    observer_provider_decl, observer_provider_extension_inline,
    observer_variant_from_pack_capability_id,
};
pub use provider::{
    ProviderDeclSpec, provider_decl, provider_extension_inline, provider_manifest,
    provider_runtime_ref, validate_provider_decl, validate_provider_extension_inline,
    validate_provider_manifest,
};
pub use state::{
    StateFixtureError, TaskStoreVariant, task_store_backend_kind, task_store_capability_id,
    task_store_capability_uri, task_store_checkpoint_key, task_store_checkpoint_path,
    task_store_operations, task_store_pack_capabilities, task_store_pack_capability_id,
    task_store_pack_manifest, task_store_pack_manifest_cbor, task_store_pending_await_path,
    task_store_provider_decl, task_store_provider_extension_inline, task_store_resume_lookup_key,
};
pub use tool::{
    ToolFixtureError, ToolVariant, tool_capability_declaration, tool_capability_id,
    tool_capability_uri, tool_operations, tool_pack_capabilities, tool_pack_capability_id,
    tool_pack_manifest, tool_pack_manifest_cbor, tool_provider_decl,
    tool_provider_extension_inline, tool_variant_from_pack_capability_id,
};

pub use greentic_cap_types as cap;
pub use greentic_cap_types::{
    CapabilityConsume, CapabilityConsumeMode, CapabilityDeclaration, CapabilityId,
    CapabilityIdError, CapabilityMetadata, CapabilityOffer, CapabilityProfile,
    CapabilityProviderRef, CapabilityRequirement, CapabilityValidationError,
};
pub use greentic_interfaces as interfaces;
pub use greentic_pack as pack;
pub use greentic_state as state_store;
pub use greentic_types::{
    CapabilitiesExtensionV1, CapabilityOfferV1, CapabilityProviderRefV1, CborError,
    ComponentCapability, ComponentManifest, ExtensionInline, ExtensionRef, Flow, PackId, PackKind,
    PackManifest, PackSignatures, ProviderDecl, ProviderExtensionInline, ProviderInstallRecord,
    ProviderManifest, ProviderRuntimeRef, StateKey, StatePath, TenantCtx,
};
