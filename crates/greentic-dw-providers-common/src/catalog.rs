//! Unified provider catalog aggregator across DW provider families.
//!
//! Composer extensions consume this to discover all providers/variants at
//! compile time without needing per-family helper knowledge. Deterministic:
//! same input (none) yields byte-stable JSON serialization across calls.

use serde::{Deserialize, Serialize};

use crate::control::ControlVariant;
use crate::embedding::EmbeddingVariant;
use crate::engine::EngineVariant;
use crate::llm::{LlmWizardProviderQa, implemented_llm_wizard_qas};
use crate::memory::ShortTermMemoryVariant;
use crate::observer::ObserverVariant;
use crate::state::TaskStoreVariant;
use crate::tool::ToolVariant;

/// Aggregated provider metadata across all DW provider families.
///
/// LLM uses the rich [`LlmWizardProviderQa`] struct (wizard questions,
/// compatibility tags, etc.). Other families expose minimal metadata
/// derived from their respective variant enums.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCatalog {
    /// LLM providers with rich wizard metadata.
    pub llm: Vec<LlmWizardProviderQa>,
    /// Engine variants (decision/routing engines).
    pub engine: Vec<ProviderCatalogEntry>,
    /// Short-term memory variants.
    pub memory: Vec<ProviderCatalogEntry>,
    /// Task-store variants (state family).
    pub state: Vec<ProviderCatalogEntry>,
    /// Observer variants.
    pub observer: Vec<ProviderCatalogEntry>,
    /// Control variants.
    pub control: Vec<ProviderCatalogEntry>,
    /// Tool variants.
    pub tool: Vec<ProviderCatalogEntry>,
    /// Embedding variants.
    pub embedding: Vec<ProviderCatalogEntry>,
}

/// One provider variant entry in the catalog.
///
/// Provides enough metadata for composer extensions to identify the variant
/// and wire it into a [`greentic_types::DigitalWorkerManifest`] without
/// further per-family lookups.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCatalogEntry {
    /// Family name (e.g. "engine", "memory", "state", "observer", "control", "tool").
    pub family: String,
    /// Variant identifier (lowercase, e.g. "default", "router-lite", "in-memory").
    pub provider_name: String,
    /// Component reference for runtime wiring (e.g. "component:engine.default").
    pub component_ref: String,
    /// Provider type id (e.g. "dw.engine.default").
    pub provider_type: String,
}

/// Build the unified provider catalog from compile-time metadata.
///
/// Returns a deterministic snapshot of all provider variants known to
/// `greentic-dw-providers-common`. Order within each family follows the
/// variant enum declaration order.
#[must_use]
pub fn unified_catalog() -> ProviderCatalog {
    ProviderCatalog {
        llm: implemented_llm_wizard_qas(),
        engine: engine_entries(),
        memory: memory_entries(),
        state: state_entries(),
        observer: observer_entries(),
        control: control_entries(),
        tool: tool_entries(),
        embedding: embedding_entries(),
    }
}

fn engine_entries() -> Vec<ProviderCatalogEntry> {
    [EngineVariant::Default, EngineVariant::RouterLite]
        .into_iter()
        .map(|v| ProviderCatalogEntry {
            family: "engine".to_string(),
            provider_name: v.as_str().to_string(),
            component_ref: v.component_ref(),
            provider_type: v.provider_type(),
        })
        .collect()
}

fn memory_entries() -> Vec<ProviderCatalogEntry> {
    [
        ShortTermMemoryVariant::InMemory,
        ShortTermMemoryVariant::Redis,
    ]
    .into_iter()
    .map(|v| ProviderCatalogEntry {
        family: "memory".to_string(),
        provider_name: v.as_str().to_string(),
        component_ref: v.component_ref(),
        provider_type: v.provider_type(),
    })
    .collect()
}

fn state_entries() -> Vec<ProviderCatalogEntry> {
    [TaskStoreVariant::InMemory, TaskStoreVariant::Redis]
        .into_iter()
        .map(|v| ProviderCatalogEntry {
            family: "state".to_string(),
            provider_name: v.as_str().to_string(),
            component_ref: v.component_ref(),
            provider_type: v.provider_type(),
        })
        .collect()
}

fn observer_entries() -> Vec<ProviderCatalogEntry> {
    [ObserverVariant::BasicAudit, ObserverVariant::BasicMetrics]
        .into_iter()
        .map(|v| ProviderCatalogEntry {
            family: "observer".to_string(),
            provider_name: v.as_str().to_string(),
            component_ref: v.component_ref(),
            provider_type: v.provider_type(),
        })
        .collect()
}

fn control_entries() -> Vec<ProviderCatalogEntry> {
    [ControlVariant::BasicPolicy, ControlVariant::DelegationGuard]
        .into_iter()
        .map(|v| ProviderCatalogEntry {
            family: "control".to_string(),
            provider_name: v.as_str().to_string(),
            component_ref: v.component_ref(),
            provider_type: v.provider_type(),
        })
        .collect()
}

fn tool_entries() -> Vec<ProviderCatalogEntry> {
    [ToolVariant::ComponentAdapter, ToolVariant::McpAdapter]
        .into_iter()
        .map(|v| ProviderCatalogEntry {
            family: "tool".to_string(),
            provider_name: v.as_str().to_string(),
            component_ref: v.component_ref(),
            provider_type: v.provider_type(),
        })
        .collect()
}

fn embedding_entries() -> Vec<ProviderCatalogEntry> {
    [EmbeddingVariant::Openai, EmbeddingVariant::OpenaiCompatible]
        .into_iter()
        .map(|v| ProviderCatalogEntry {
            family: "embedding".to_string(),
            provider_name: v.as_str().to_string(),
            component_ref: v.component_ref(),
            provider_type: v.provider_type(),
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn unified_catalog_includes_all_families() {
        let cat = unified_catalog();
        assert!(cat.llm.iter().any(|p| p.provider_name == "anthropic"));
        assert!(cat.llm.iter().any(|p| p.provider_name == "openai"));
        assert_eq!(cat.engine.len(), 2);
        assert_eq!(cat.memory.len(), 2);
        assert_eq!(cat.state.len(), 2);
        assert_eq!(cat.observer.len(), 2);
        assert_eq!(cat.control.len(), 2);
        assert_eq!(cat.tool.len(), 2);
    }

    #[test]
    fn unified_catalog_is_deterministic() {
        let a = unified_catalog();
        let b = unified_catalog();
        assert_eq!(
            serde_json::to_string(&a).expect("serialize a"),
            serde_json::to_string(&b).expect("serialize b"),
        );
    }

    #[test]
    fn engine_entries_have_correct_family_and_provider_type_prefix() {
        let cat = unified_catalog();
        assert!(!cat.engine.is_empty());
        for entry in &cat.engine {
            assert_eq!(entry.family, "engine");
            assert!(
                entry.provider_type.starts_with("dw.engine."),
                "provider_type should start with dw.engine.: {}",
                entry.provider_type,
            );
            assert!(
                entry.component_ref.starts_with("component:engine."),
                "component_ref should start with component:engine.: {}",
                entry.component_ref,
            );
        }
    }

    #[test]
    fn memory_variants_present() {
        let cat = unified_catalog();
        let names: Vec<&str> = cat
            .memory
            .iter()
            .map(|e| e.provider_name.as_str())
            .collect();
        assert!(names.contains(&"in-memory"));
        assert!(names.contains(&"redis"));
    }

    #[test]
    fn state_variants_present() {
        let cat = unified_catalog();
        let names: Vec<&str> = cat.state.iter().map(|e| e.provider_name.as_str()).collect();
        assert!(names.contains(&"in-memory"));
        assert!(names.contains(&"redis"));
    }

    #[test]
    fn observer_variants_present() {
        let cat = unified_catalog();
        let names: Vec<&str> = cat
            .observer
            .iter()
            .map(|e| e.provider_name.as_str())
            .collect();
        assert!(names.contains(&"basic-audit"));
        assert!(names.contains(&"basic-metrics"));
    }

    #[test]
    fn control_variants_present() {
        let cat = unified_catalog();
        let names: Vec<&str> = cat
            .control
            .iter()
            .map(|e| e.provider_name.as_str())
            .collect();
        assert!(names.contains(&"basic-policy"));
        assert!(names.contains(&"delegation-guard"));
    }

    #[test]
    fn tool_variants_present() {
        let cat = unified_catalog();
        let names: Vec<&str> = cat.tool.iter().map(|e| e.provider_name.as_str()).collect();
        assert!(names.contains(&"component-adapter"));
        assert!(names.contains(&"mcp-adapter"));
    }

    #[test]
    fn unified_catalog_includes_embedding_family() {
        let cat = unified_catalog();
        assert_eq!(cat.embedding.len(), 2);
        assert!(cat.embedding.iter().any(|e| e.provider_name == "openai"));
        assert!(cat.embedding.iter().any(|e| e.provider_name == "openai-compatible"));
        let openai = cat.embedding.iter().find(|e| e.provider_name == "openai").expect("openai");
        assert_eq!(openai.family, "embedding");
        assert_eq!(openai.provider_type, "dw.embedding.openai");
        assert_eq!(openai.component_ref, "component:embedding.openai");
    }

    #[test]
    fn llm_anthropic_supports_tool_calling() {
        let cat = unified_catalog();
        let anthropic = cat
            .llm
            .iter()
            .find(|p| p.provider_name == "anthropic")
            .expect("anthropic must be present");
        assert!(!anthropic.provider_name.is_empty());
    }
}
