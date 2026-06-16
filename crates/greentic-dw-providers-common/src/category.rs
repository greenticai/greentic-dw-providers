use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

/// Provider categories planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderCategory {
    /// Engine providers.
    Engine,
    /// LLM providers.
    Llm,
    /// Memory providers.
    Memory,
    /// State providers.
    State,
    /// Control providers.
    Control,
    /// Observer providers.
    Observer,
    /// Tool providers.
    Tool,
    /// Embedding providers.
    Embedding,
}

impl ProviderCategory {
    /// Returns the category as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::Llm => "llm",
            Self::Memory => "memory",
            Self::State => "state",
            Self::Control => "control",
            Self::Observer => "observer",
            Self::Tool => "tool",
            Self::Embedding => "embedding",
        }
    }

    /// Returns the canonical provider type for a provider name in this category.
    #[must_use]
    pub fn provider_type(self, provider_name: impl AsRef<str>) -> String {
        format!("dw.{}.{}", self.as_str(), provider_name.as_ref())
    }
}

/// Returns the canonical provider type for a provider name in a category.
#[must_use]
pub fn provider_type(category: ProviderCategory, provider_name: impl AsRef<str>) -> String {
    category.provider_type(provider_name)
}

impl fmt::Display for ProviderCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderCategory {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "engine" => Ok(Self::Engine),
            "llm" => Ok(Self::Llm),
            "memory" => Ok(Self::Memory),
            "state" => Ok(Self::State),
            "control" => Ok(Self::Control),
            "observer" => Ok(Self::Observer),
            "tool" => Ok(Self::Tool),
            "embedding" => Ok(Self::Embedding),
            _ => Err("unknown provider category"),
        }
    }
}

/// Returns the workspace version used by the root package.
#[must_use]
pub const fn workspace_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the DW capability URI for a provider category and local capability name.
#[must_use]
pub fn capability_uri(category: ProviderCategory, capability: impl AsRef<str>) -> String {
    format!("cap://dw.{}.{}", category.as_str(), capability.as_ref())
}

/// Returns the planned provider categories in workspace order.
#[must_use]
pub const fn planned_categories() -> [ProviderCategory; 8] {
    [
        ProviderCategory::Engine,
        ProviderCategory::Llm,
        ProviderCategory::Memory,
        ProviderCategory::State,
        ProviderCategory::Control,
        ProviderCategory::Observer,
        ProviderCategory::Tool,
        ProviderCategory::Embedding,
    ]
}

/// Returns a short banner describing the current scaffold.
#[must_use]
pub fn workspace_banner() -> String {
    let categories = planned_categories()
        .into_iter()
        .map(ProviderCategory::as_str)
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "greentic-dw-providers {} scaffold ({categories})",
        workspace_version()
    )
}
