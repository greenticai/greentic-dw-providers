//! Knowledge family metadata helpers.

use semver::Version;

use crate::category::ProviderCategory;
use crate::{
    CapabilityIdError, ProviderDeclSpec, capability_declaration, capability_provider_ref,
    provider_decl,
};

use greentic_cap_types::CapabilityId;
use greentic_types::{
    CapabilitiesExtensionError, CapabilitiesExtensionV1, CapabilityOfferV1,
    CapabilityProviderRefV1, CborError, PackId, PackKind, PackManifest, PackSignatures,
    encode_pack_manifest,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CapabilityDeclaration;

/// Knowledge (document-RAG) provider variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KnowledgeVariant {
    /// Chronicle-backed document-RAG implementation.
    Chronicle,
}

impl KnowledgeVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chronicle => "chronicle",
        }
    }

    /// Returns the component reference for runtime wiring.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:knowledge.{}", self.as_str())
    }

    /// Returns the provider type identifier (e.g. `dw.knowledge.chronicle`).
    #[must_use]
    pub fn provider_type(self) -> String {
        ProviderCategory::Knowledge.provider_type(self.as_str())
    }
}

/// Errors produced while building knowledge fixtures.
#[derive(Debug, Error)]
pub enum KnowledgeFixtureError {
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

/// Returns the knowledge capability URI (`cap://dw.knowledge`).
#[must_use]
pub fn knowledge_capability_uri() -> String {
    format!("cap://dw.{}", ProviderCategory::Knowledge.as_str())
}

/// Returns the knowledge pack capability identifier (`greentic.cap.knowledge`).
#[must_use]
pub fn knowledge_pack_capability_id() -> String {
    format!("greentic.cap.{}", ProviderCategory::Knowledge.as_str())
}

/// Returns the operations exposed by a knowledge provider.
#[must_use]
pub const fn knowledge_operations() -> [&'static str; 2] {
    ["knowledge.ingest", "knowledge.search"]
}

/// Returns the canonical provider declaration for a knowledge backend.
#[must_use]
pub fn knowledge_provider_decl(variant: KnowledgeVariant) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Knowledge,
        provider_name: variant.as_str().to_string(),
        capabilities: vec![knowledge_pack_capability_id()],
        ops: knowledge_operations()
            .into_iter()
            .map(str::to_string)
            .collect(),
        config_schema_ref: format!("schemas/knowledge/{}.json", variant.as_str()),
        state_schema_ref: None,
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!("docs/providers/knowledge/{}.md", variant.as_str())),
    })
}

/// Returns the knowledge capability declaration for a provider variant.
///
/// Uses the exact URI `cap://dw.knowledge` (no trailing dot) rather than the
/// `capability_id(cat, "")` helper which would append a dot.
pub fn knowledge_capability_declaration(
    variant: KnowledgeVariant,
) -> Result<CapabilityDeclaration, KnowledgeFixtureError> {
    let capability = CapabilityId::new(knowledge_capability_uri())
        .map_err(KnowledgeFixtureError::CapabilityId)?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.knowledge.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        "knowledge.ingest",
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability extension payload for a knowledge backend.
///
/// Uses `knowledge_pack_capability_id()` directly (`greentic.cap.knowledge`,
/// no trailing dot) rather than the generic `pack_capabilities_extension` helper.
#[must_use]
pub fn knowledge_pack_capabilities(variant: KnowledgeVariant) -> CapabilitiesExtensionV1 {
    CapabilitiesExtensionV1::new(vec![CapabilityOfferV1 {
        offer_id: format!("offer.knowledge.{}", variant.as_str()),
        cap_id: knowledge_pack_capability_id(),
        version: "v1".to_string(),
        provider: CapabilityProviderRefV1 {
            component_ref: variant.component_ref(),
            op: "knowledge.ingest".to_string(),
        },
        scope: None,
        priority: 0,
        requires_setup: false,
        setup: None,
        applies_to: None,
    }])
}

/// Builds a pack manifest for a knowledge backend.
pub fn knowledge_pack_manifest(
    pack_id: PackId,
    variant: KnowledgeVariant,
) -> Result<PackManifest, KnowledgeFixtureError> {
    let provider_decl = knowledge_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("knowledge {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: knowledge_pack_capability_id(),
            description: Some("knowledge (document-RAG) capability".to_string()),
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
        .set_capabilities_extension_v1(knowledge_pack_capabilities(variant))
        .map_err(KnowledgeFixtureError::from)?;
    Ok(manifest)
}

/// Encodes a knowledge pack manifest to CBOR bytes.
pub fn knowledge_pack_manifest_cbor(
    pack_id: PackId,
    variant: KnowledgeVariant,
) -> Result<Vec<u8>, KnowledgeFixtureError> {
    let manifest = knowledge_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn capability_uri_is_exact() {
        assert_eq!(knowledge_capability_uri(), "cap://dw.knowledge");
    }

    #[test]
    fn pack_capability_id_is_exact() {
        assert_eq!(knowledge_pack_capability_id(), "greentic.cap.knowledge");
    }

    #[test]
    fn variant_provider_type() {
        assert_eq!(
            KnowledgeVariant::Chronicle.provider_type(),
            "dw.knowledge.chronicle"
        );
    }

    #[test]
    fn variant_component_ref() {
        assert_eq!(
            KnowledgeVariant::Chronicle.component_ref(),
            "component:knowledge.chronicle"
        );
    }

    #[test]
    fn knowledge_operations_has_two_entries() {
        let ops = knowledge_operations();
        assert_eq!(ops.len(), 2);
        assert!(ops.contains(&"knowledge.ingest"));
        assert!(ops.contains(&"knowledge.search"));
    }
}
