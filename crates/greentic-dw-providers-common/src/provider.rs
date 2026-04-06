use std::collections::BTreeMap;

use greentic_types::{ErrorCode, GResult, GreenticError};

use crate::{ProviderCategory, provider_type};

use greentic_types::{ProviderDecl, ProviderExtensionInline, ProviderManifest, ProviderRuntimeRef};

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn validate_provider_decl_tail(decl: &ProviderDecl) -> GResult<()> {
    if decl.config_schema_ref.trim().is_empty() {
        return Err(invalid("ProviderDecl.config_schema_ref must not be empty"));
    }

    for capability in &decl.capabilities {
        if capability.trim().is_empty() {
            return Err(invalid(
                "ProviderDecl.capabilities must not contain empty entries",
            ));
        }
    }

    for operation in &decl.ops {
        if operation.trim().is_empty() {
            return Err(invalid("ProviderDecl.ops must not contain empty entries"));
        }
    }

    if let Some(docs_ref) = &decl.docs_ref
        && docs_ref.trim().is_empty()
    {
        return Err(invalid(
            "ProviderDecl.docs_ref must not be empty when present",
        ));
    }

    Ok(())
}

/// Input parameters for constructing a [`ProviderDecl`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderDeclSpec {
    /// Provider category.
    pub category: ProviderCategory,
    /// Provider type name within the category.
    pub provider_name: String,
    /// Capabilities advertised by the provider.
    pub capabilities: Vec<String>,
    /// Operations exposed by the provider.
    pub ops: Vec<String>,
    /// JSON schema reference for provider configuration.
    pub config_schema_ref: String,
    /// Optional JSON schema reference for provider state.
    pub state_schema_ref: Option<String>,
    /// Component reference for the provider runtime.
    pub component_ref: String,
    /// Export name for the runtime.
    pub export: String,
    /// WIT world for the runtime.
    pub world: String,
    /// Optional documentation reference.
    pub docs_ref: Option<String>,
}

/// Creates a provider runtime binding reference.
#[must_use]
pub fn provider_runtime_ref(
    component_ref: impl Into<String>,
    export: impl Into<String>,
    world: impl Into<String>,
) -> ProviderRuntimeRef {
    ProviderRuntimeRef {
        component_ref: component_ref.into(),
        export: export.into(),
        world: world.into(),
    }
}

/// Creates a provider manifest for a named provider in a category.
#[must_use]
pub fn provider_manifest(
    category: ProviderCategory,
    provider_name: impl AsRef<str>,
    capabilities: Vec<String>,
    ops: Vec<String>,
    config_schema_ref: Option<String>,
    state_schema_ref: Option<String>,
) -> ProviderManifest {
    ProviderManifest {
        provider_type: provider_type(category, provider_name),
        capabilities,
        ops,
        config_schema_ref,
        state_schema_ref,
    }
}

/// Creates a provider declaration with runtime metadata.
#[must_use]
pub fn provider_decl(spec: ProviderDeclSpec) -> ProviderDecl {
    ProviderDecl {
        provider_type: provider_type(spec.category, &spec.provider_name),
        capabilities: spec.capabilities,
        ops: spec.ops,
        config_schema_ref: spec.config_schema_ref,
        state_schema_ref: spec.state_schema_ref,
        runtime: provider_runtime_ref(spec.component_ref, spec.export, spec.world),
        docs_ref: spec.docs_ref,
    }
}

/// Creates an inline provider extension payload.
#[must_use]
pub fn provider_extension_inline(providers: Vec<ProviderDecl>) -> ProviderExtensionInline {
    ProviderExtensionInline {
        providers,
        additional_fields: BTreeMap::new(),
    }
}

/// Validates a provider manifest for basic completeness.
pub fn validate_provider_manifest(manifest: &ProviderManifest) -> GResult<()> {
    if manifest.provider_type.trim().is_empty() {
        return Err(invalid("ProviderManifest.provider_type must not be empty"));
    }

    for capability in &manifest.capabilities {
        if capability.trim().is_empty() {
            return Err(invalid(
                "ProviderManifest.capabilities must not contain empty entries",
            ));
        }
    }

    for operation in &manifest.ops {
        if operation.trim().is_empty() {
            return Err(invalid(
                "ProviderManifest.ops must not contain empty entries",
            ));
        }
    }

    if let Some(schema_ref) = &manifest.config_schema_ref
        && schema_ref.trim().is_empty()
    {
        return Err(invalid(
            "ProviderManifest.config_schema_ref must not be empty when present",
        ));
    }

    if let Some(schema_ref) = &manifest.state_schema_ref
        && schema_ref.trim().is_empty()
    {
        return Err(invalid(
            "ProviderManifest.state_schema_ref must not be empty when present",
        ));
    }

    Ok(())
}

/// Validates a provider declaration for basic completeness.
pub fn validate_provider_decl(decl: &ProviderDecl) -> GResult<()> {
    if decl.provider_type.trim().is_empty() {
        return Err(invalid("ProviderDecl.provider_type must not be empty"));
    }

    if decl.runtime.component_ref.trim().is_empty()
        || decl.runtime.export.trim().is_empty()
        || decl.runtime.world.trim().is_empty()
    {
        return Err(invalid("ProviderDecl.runtime fields must not be empty"));
    }

    validate_provider_decl_tail(decl)
}

/// Validates an inline provider extension using the shared Greentic provider model.
pub fn validate_provider_extension_inline(extension: &ProviderExtensionInline) -> GResult<()> {
    extension.validate_basic()?;
    for provider in &extension.providers {
        validate_provider_decl_tail(provider)?;
    }
    Ok(())
}
