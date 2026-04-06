use semver::Version;
use thiserror::Error;

use crate::{
    CapabilityDeclaration, CapabilityIdError, ProviderCategory, ProviderDeclSpec,
    capability_declaration, capability_id, capability_provider_ref, pack_capabilities_extension,
    provider_decl, provider_extension_inline,
};

use greentic_types::{
    CapabilitiesExtensionError, CborError, PackId, PackKind, PackManifest, PackSignatures,
    ProviderExtensionInline, encode_pack_manifest,
};

/// Tool provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolVariant {
    /// WASM adapter.
    WasmAdapter,
    /// MCP adapter.
    McpAdapter,
}

impl ToolVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WasmAdapter => "wasm-adapter",
            Self::McpAdapter => "mcp-adapter",
        }
    }

    /// Returns the canonical capability name for the variant.
    #[must_use]
    pub const fn capability_name(self) -> &'static str {
        match self {
            Self::WasmAdapter => "wasm",
            Self::McpAdapter => "mcp",
        }
    }

    /// Returns the provider component reference used by this variant.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:tool.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(ProviderCategory::Tool, self.as_str())
    }
}

/// Errors produced while building tool fixtures.
#[derive(Debug, Error)]
pub enum ToolFixtureError {
    /// Capability identifier parsing failed.
    #[error(transparent)]
    CapabilityId(#[from] CapabilityIdError),
    /// Pack capability extension construction failed.
    #[error(transparent)]
    CapabilitiesExtension(#[from] CapabilitiesExtensionError),
    /// Pack CBOR encoding failed.
    #[error(transparent)]
    Cbor(#[from] CborError),
}

/// Returns the canonical tool capability URI for a local capability name.
#[must_use]
pub fn tool_capability_uri(capability: impl AsRef<str>) -> String {
    crate::capability_uri(ProviderCategory::Tool, capability)
}

/// Returns the canonical tool capability identifier.
pub fn tool_capability_id(
    capability: impl AsRef<str>,
) -> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    capability_id(ProviderCategory::Tool, capability)
}

/// Returns the canonical tool pack capability identifier.
#[must_use]
pub fn tool_pack_capability_id(capability: impl AsRef<str>) -> String {
    crate::pack_capability_id(ProviderCategory::Tool, capability)
}

/// Resolves a tool variant from a pack capability id.
#[must_use]
pub fn tool_variant_from_pack_capability_id(capability_id: impl AsRef<str>) -> Option<ToolVariant> {
    match capability_id.as_ref() {
        "greentic.cap.tool.wasm" => Some(ToolVariant::WasmAdapter),
        "greentic.cap.tool.mcp" => Some(ToolVariant::McpAdapter),
        _ => None,
    }
}

/// Returns the operations expected from a tool provider.
#[must_use]
pub const fn tool_operations() -> [&'static str; 2] {
    ["tool.invoke", "tool.describe"]
}

fn tool_operation_for_variant(variant: ToolVariant) -> &'static str {
    match variant {
        ToolVariant::WasmAdapter => "tool.invoke",
        ToolVariant::McpAdapter => "tool.describe",
    }
}

/// Returns the canonical provider declaration for a tool backend.
#[must_use]
pub fn tool_provider_decl(variant: ToolVariant) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Tool,
        provider_name: variant.as_str().to_string(),
        capabilities: vec![tool_pack_capability_id(variant.capability_name())],
        ops: tool_operations().into_iter().map(str::to_string).collect(),
        config_schema_ref: format!("schemas/tool/{}.json", variant.as_str()),
        state_schema_ref: None,
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!("docs/providers/tool/{}.md", variant.as_str())),
    })
}

/// Returns the canonical provider extension payload for a tool backend.
#[must_use]
pub fn tool_provider_extension_inline(variants: Vec<ToolVariant>) -> ProviderExtensionInline {
    let providers = variants.into_iter().map(tool_provider_decl).collect();
    provider_extension_inline(providers)
}

/// Returns the tool capability declaration for a provider variant.
pub fn tool_capability_declaration(
    variant: ToolVariant,
) -> Result<CapabilityDeclaration, ToolFixtureError> {
    let capability = tool_capability_id(variant.capability_name())?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.tool.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        tool_operation_for_variant(variant),
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability payload for a tool backend.
#[must_use]
pub fn tool_pack_capabilities(variant: ToolVariant) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::Tool,
        variant.capability_name(),
        format!("offer.tool.{}", variant.as_str()),
        variant.component_ref(),
        tool_operation_for_variant(variant),
    )
}

/// Builds a pack manifest for a tool backend.
pub fn tool_pack_manifest(
    pack_id: PackId,
    variant: ToolVariant,
) -> Result<PackManifest, ToolFixtureError> {
    let provider_decl = tool_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("tool {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: tool_pack_capability_id(variant.capability_name()),
            description: Some("tool capability".to_string()),
        }],
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };

    manifest
        .ensure_provider_extension_inline()
        .providers
        .push(provider_decl);
    manifest
        .set_capabilities_extension_v1(tool_pack_capabilities(variant))
        .map_err(ToolFixtureError::from)?;
    Ok(manifest)
}

/// Encodes a tool pack manifest to CBOR bytes.
pub fn tool_pack_manifest_cbor(
    pack_id: PackId,
    variant: ToolVariant,
) -> Result<Vec<u8>, ToolFixtureError> {
    let manifest = tool_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}
