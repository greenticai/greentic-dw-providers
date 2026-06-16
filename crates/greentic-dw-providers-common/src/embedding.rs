//! Embedding family metadata helpers (variant enum + capability re-exports).

pub use greentic_dw_embedding::{EMBEDDING_CAPABILITY_URI, EMBEDDING_PACK_CAPABILITY_ID};

use serde::{Deserialize, Serialize};

use crate::category::ProviderCategory;

/// Implemented embedding backends, in catalog declaration order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EmbeddingVariant {
    /// Native OpenAI embedding backend (https only).
    Openai,
    /// OpenAI-compatible embedding backend (http/https, required `base_url`).
    OpenaiCompatible,
}

impl EmbeddingVariant {
    /// Returns the variant as a lowercase kebab-case string identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::OpenaiCompatible => "openai-compatible",
        }
    }

    /// Returns the component reference for runtime wiring.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:embedding.{}", self.as_str())
    }

    /// Returns the provider type identifier (e.g. `dw.embedding.openai`).
    #[must_use]
    pub fn provider_type(self) -> String {
        ProviderCategory::Embedding.provider_type(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{capability::pack_capability_id, category::capability_uri};

    #[test]
    fn capability_strings_follow_convention() {
        assert_eq!(
            capability_uri(ProviderCategory::Embedding, "default"),
            "cap://dw.embedding.default"
        );
        assert_eq!(
            pack_capability_id(ProviderCategory::Embedding, "default"),
            "greentic.cap.embedding.default"
        );
    }

    #[test]
    fn variant_metadata() {
        assert_eq!(EmbeddingVariant::OpenaiCompatible.as_str(), "openai-compatible");
        assert_eq!(EmbeddingVariant::Openai.component_ref(), "component:embedding.openai");
        assert_eq!(EmbeddingVariant::Openai.provider_type(), "dw.embedding.openai");
    }
}
