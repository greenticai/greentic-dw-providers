//! Configuration for the Chronicle-backed long-term memory provider.

use std::fmt;

/// Default Neo4j database name when none is supplied.
pub const DEFAULT_NEO4J_DATABASE: &str = "neo4j";

/// Default concurrency cap passed to the Chronicle engine.
pub const DEFAULT_MAX_CONCURRENCY: usize = 20;

/// Default number of facts returned by [`recall`](crate::ChronicleLongTermMemory) when
/// the caller does not specify a limit.
pub const DEFAULT_RECALL_LIMIT: usize = 10;

/// Connection and model configuration for [`ChronicleLongTermMemory`].
///
/// Secrets ([`neo4j_password`](Self::neo4j_password) and
/// [`openai_api_key`](Self::openai_api_key)) are redacted by the manual [`fmt::Debug`]
/// implementation so they never leak into logs.
#[derive(Clone)]
pub struct ChronicleMemoryConfig {
    /// Bolt URI of the Neo4j instance (for example `neo4j://localhost:7687`).
    pub neo4j_uri: String,
    /// Neo4j username.
    pub neo4j_user: String,
    /// Neo4j password. Redacted in [`fmt::Debug`].
    pub neo4j_password: String,
    /// Neo4j database name. Defaults to [`DEFAULT_NEO4J_DATABASE`].
    pub neo4j_database: String,
    /// OpenAI (or compatible) API key. When `None`, the underlying client reads
    /// `OPENAI_API_KEY` from the environment. Redacted in [`fmt::Debug`].
    pub openai_api_key: Option<String>,
    /// Override base URL for OpenAI-compatible endpoints.
    pub openai_base_url: Option<String>,
    /// Primary model name. When `None`, the Chronicle OpenAI client default applies.
    pub model: Option<String>,
    /// Small model name used for cheaper extraction calls. When `None`, the
    /// Chronicle OpenAI client default applies.
    pub small_model: Option<String>,
    /// Embedding model name. When `None`, the Chronicle embedder default applies.
    pub embedding_model: Option<String>,
    /// Embedding dimension. When `None`, the Chronicle embedder default applies.
    pub embedding_dim: Option<usize>,
    /// Concurrency cap for the Chronicle engine. Defaults to [`DEFAULT_MAX_CONCURRENCY`].
    pub max_concurrency: usize,
    /// Default recall limit. Defaults to [`DEFAULT_RECALL_LIMIT`].
    pub recall_limit: usize,
}

impl ChronicleMemoryConfig {
    /// Builds a config from the required Neo4j connection trio, applying defaults
    /// to every other field.
    pub fn new(
        neo4j_uri: impl Into<String>,
        neo4j_user: impl Into<String>,
        neo4j_password: impl Into<String>,
    ) -> Self {
        Self {
            neo4j_uri: neo4j_uri.into(),
            neo4j_user: neo4j_user.into(),
            neo4j_password: neo4j_password.into(),
            neo4j_database: DEFAULT_NEO4J_DATABASE.to_string(),
            openai_api_key: None,
            openai_base_url: None,
            model: None,
            small_model: None,
            embedding_model: None,
            embedding_dim: None,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            recall_limit: DEFAULT_RECALL_LIMIT,
        }
    }

    /// Sets the Neo4j database name.
    #[must_use]
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.neo4j_database = database.into();
        self
    }

    /// Sets the OpenAI API key.
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.openai_api_key = Some(api_key.into());
        self
    }

    /// Sets the OpenAI-compatible base URL.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.openai_base_url = Some(base_url.into());
        self
    }

    /// Sets the primary and small model names.
    #[must_use]
    pub fn with_models(mut self, model: Option<String>, small_model: Option<String>) -> Self {
        self.model = model;
        self.small_model = small_model;
        self
    }

    /// Sets the embedding model name and dimension.
    #[must_use]
    pub fn with_embedding(
        mut self,
        embedding_model: Option<String>,
        embedding_dim: Option<usize>,
    ) -> Self {
        self.embedding_model = embedding_model;
        self.embedding_dim = embedding_dim;
        self
    }

    /// Sets the engine concurrency cap.
    #[must_use]
    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = max_concurrency;
        self
    }

    /// Sets the default recall limit.
    #[must_use]
    pub fn with_recall_limit(mut self, recall_limit: usize) -> Self {
        self.recall_limit = recall_limit;
        self
    }
}

/// Placeholder string substituted for secret fields in [`fmt::Debug`] output.
const REDACTED: &str = "<redacted>";

impl fmt::Debug for ChronicleMemoryConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChronicleMemoryConfig")
            .field("neo4j_uri", &self.neo4j_uri)
            .field("neo4j_user", &self.neo4j_user)
            .field("neo4j_password", &REDACTED)
            .field("neo4j_database", &self.neo4j_database)
            .field(
                "openai_api_key",
                &self.openai_api_key.as_ref().map(|_| REDACTED),
            )
            .field("openai_base_url", &self.openai_base_url)
            .field("model", &self.model)
            .field("small_model", &self.small_model)
            .field("embedding_model", &self.embedding_model)
            .field("embedding_dim", &self.embedding_dim)
            .field("max_concurrency", &self.max_concurrency)
            .field("recall_limit", &self.recall_limit)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_applies_defaults() {
        let cfg = ChronicleMemoryConfig::new("neo4j://localhost:7687", "neo4j", "secret-pw");
        assert_eq!(cfg.neo4j_database, DEFAULT_NEO4J_DATABASE);
        assert_eq!(cfg.max_concurrency, DEFAULT_MAX_CONCURRENCY);
        assert_eq!(cfg.recall_limit, DEFAULT_RECALL_LIMIT);
        assert!(cfg.openai_api_key.is_none());
    }

    #[test]
    fn debug_redacts_secrets() {
        let cfg = ChronicleMemoryConfig::new("neo4j://localhost:7687", "neo4j", "super-secret-pw")
            .with_api_key("sk-very-secret-key");
        let rendered = format!("{cfg:?}");

        assert!(rendered.contains(REDACTED), "expected redaction marker");
        assert!(
            !rendered.contains("super-secret-pw"),
            "neo4j password must not appear in debug output"
        );
        assert!(
            !rendered.contains("sk-very-secret-key"),
            "openai api key must not appear in debug output"
        );
        // Non-secret fields remain visible.
        assert!(rendered.contains("neo4j://localhost:7687"));
    }

    #[test]
    fn debug_redacts_present_api_key_only_as_marker() {
        let no_key = ChronicleMemoryConfig::new("uri", "user", "pw");
        let rendered = format!("{no_key:?}");
        // With no key, the option renders as None (not the redaction marker).
        assert!(rendered.contains("openai_api_key: None"));
    }
}
