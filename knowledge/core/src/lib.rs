#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Normalized knowledge (document-RAG) provider contract.
//!
//! This crate defines the [`Knowledge`] trait and the data-transfer objects
//! shared by all knowledge backends. No concrete backend or chronicle types
//! are present here; those live in sibling crates.

use async_trait::async_trait;
use greentic_types::TenantCtx;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

pub use greentic_dw_providers_common::knowledge::{
    knowledge_capability_uri, knowledge_pack_capability_id,
};

// ─── DTOs ───────────────────────────────────────────────────────────────────

/// One pre-chunked unit to ingest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    /// Backend-assigned or caller-supplied document identifier.
    pub doc_id: String,
    /// Position of this chunk within the source document (zero-based).
    pub chunk_index: usize,
    /// Text content of this chunk.
    pub text: String,
    /// Arbitrary metadata attached to the chunk.
    #[serde(default)]
    pub metadata: Map<String, Value>,
    /// Optional pre-computed embedding vector for this chunk.
    ///
    /// When present, backends should use this vector directly instead of
    /// computing one from `text`. `None` preserves the existing
    /// backend-computed-embedding behavior. `#[serde(default)]` keeps
    /// payloads without this field decoding to `None`.
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
}

/// Outcome of an ingest call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IngestOutcome {
    /// Backend-assigned identifiers for the stored chunks.
    pub chunk_ids: Vec<String>,
}

/// A retrieval query.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    /// Natural-language query or keyword phrase.
    pub query: String,
    /// Maximum number of hits to return. `None` lets the backend decide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// A retrieval hit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RetrievedChunk {
    /// Text of the retrieved chunk.
    pub text: String,
    /// Relevance score (higher is more relevant).
    pub score: f64,
    /// Document identifier, if provided by the backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
    /// Chunk index within the source document, if provided by the backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_index: Option<usize>,
    /// Metadata attached to the retrieved chunk.
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

// ─── Error ──────────────────────────────────────────────────────────────────

/// Errors returned by knowledge operations.
#[derive(Debug, Error)]
pub enum KnowledgeError {
    /// The underlying storage or RAG backend returned an error.
    #[error("knowledge backend error: {0}")]
    Backend(String),
    /// The tenant identifier is invalid or unknown.
    #[error("invalid tenant: {0}")]
    InvalidTenant(String),
    /// No knowledge backend has been configured for this tenant.
    #[error("knowledge provider not configured")]
    NotConfigured,
}

/// Convenience type alias for knowledge operation results.
pub type KnowledgeResult<T> = Result<T, KnowledgeError>;

// ─── Trait ──────────────────────────────────────────────────────────────────

/// Contract implemented by knowledge (document-RAG) backends.
///
/// Implementations are expected to:
/// * ingest pre-chunked document text during [`ingest`], associating it with the tenant corpus;
/// * support hybrid retrieval so that [`search`] can rank and return relevant chunks;
/// * scope all storage and retrieval strictly to the provided [`TenantCtx`].
///
/// The knowledge `group_id` (corpus isolation key) is derived from the tenant by
/// the provider implementation — callers need not supply it directly.
#[async_trait]
pub trait Knowledge: Send + Sync {
    /// Ingest a batch of pre-chunked document text into the knowledge corpus.
    ///
    /// Returns an [`IngestOutcome`] containing the backend-assigned identifiers
    /// for each stored chunk.
    async fn ingest(
        &self,
        tenant: &TenantCtx,
        chunks: Vec<KnowledgeChunk>,
    ) -> KnowledgeResult<IngestOutcome>;

    /// Retrieve chunks relevant to the given query from the tenant's knowledge corpus.
    ///
    /// Returns a ranked list of [`RetrievedChunk`] entries ordered by relevance
    /// descending. The result set is bounded by `query.limit` when provided.
    async fn search(
        &self,
        tenant: &TenantCtx,
        query: KnowledgeQuery,
    ) -> KnowledgeResult<Vec<RetrievedChunk>>;
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use greentic_types::{EnvId, TenantId};

    fn tenant(value: &str) -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from(value).expect("tenant id"),
        )
    }

    // ── capability const tests ─────────────────────────────────────────────

    #[test]
    fn capability_uri_is_exact() {
        assert_eq!(knowledge_capability_uri(), "cap://dw.knowledge");
    }

    #[test]
    fn pack_capability_id_is_exact() {
        assert_eq!(knowledge_pack_capability_id(), "greentic.cap.knowledge");
    }

    // ── serde roundtrips ───────────────────────────────────────────────────

    #[test]
    fn knowledge_chunk_roundtrips() {
        let mut metadata = Map::new();
        metadata.insert("source".to_string(), Value::String("readme.md".to_string()));

        let chunk = KnowledgeChunk {
            doc_id: "doc-001".to_string(),
            chunk_index: 2,
            text: "Rust is a systems programming language.".to_string(),
            metadata,
            embedding: None,
        };

        let json = serde_json::to_string(&chunk).expect("serialize");
        let back: KnowledgeChunk = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.doc_id, chunk.doc_id);
        assert_eq!(back.chunk_index, chunk.chunk_index);
        assert_eq!(back.text, chunk.text);
        assert_eq!(
            back.metadata["source"],
            Value::String("readme.md".to_string())
        );
    }

    #[test]
    fn knowledge_chunk_empty_metadata_default() {
        let json = r#"{"doc_id":"d","chunk_index":0,"text":"t"}"#;
        let chunk: KnowledgeChunk = serde_json::from_str(json).expect("deserialize");
        assert!(chunk.metadata.is_empty());
        assert_eq!(chunk.embedding, None, "embedding should default to None");
    }

    #[test]
    fn knowledge_chunk_with_precomputed_embedding_roundtrips() {
        let chunk = KnowledgeChunk {
            doc_id: "doc-002".to_string(),
            chunk_index: 0,
            text: "Precomputed vector chunk.".to_string(),
            metadata: Map::new(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
        };

        let json = serde_json::to_string(&chunk).expect("serialize");
        let back: KnowledgeChunk = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.embedding, Some(vec![0.1, 0.2, 0.3]));
    }

    #[test]
    fn ingest_outcome_roundtrips() {
        let outcome = IngestOutcome {
            chunk_ids: vec!["id-1".to_string(), "id-2".to_string()],
        };
        let json = serde_json::to_string(&outcome).expect("serialize");
        let back: IngestOutcome = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.chunk_ids, outcome.chunk_ids);
    }

    #[test]
    fn knowledge_query_roundtrips_with_limit() {
        let q = KnowledgeQuery {
            query: "what is Rust?".to_string(),
            limit: Some(5),
        };
        let json = serde_json::to_string(&q).expect("serialize");
        let back: KnowledgeQuery = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.query, q.query);
        assert_eq!(back.limit, Some(5));
    }

    #[test]
    fn knowledge_query_limit_omitted_when_none() {
        let q = KnowledgeQuery {
            query: "test".to_string(),
            limit: None,
        };
        let json = serde_json::to_string(&q).expect("serialize");
        assert!(!json.contains("limit"), "limit should be omitted: {json}");
    }

    #[test]
    fn retrieved_chunk_roundtrips() {
        let chunk = RetrievedChunk {
            text: "Rust is fast.".to_string(),
            score: 0.95,
            doc_id: Some("doc-001".to_string()),
            chunk_index: Some(3),
            metadata: Map::new(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize");
        let back: RetrievedChunk = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.text, chunk.text);
        assert!((back.score - 0.95).abs() < f64::EPSILON);
        assert_eq!(back.doc_id, Some("doc-001".to_string()));
        assert_eq!(back.chunk_index, Some(3));
    }

    #[test]
    fn retrieved_chunk_optional_fields_omitted_when_none() {
        let chunk = RetrievedChunk {
            text: "t".to_string(),
            score: 0.5,
            doc_id: None,
            chunk_index: None,
            metadata: Map::new(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize");
        assert!(!json.contains("doc_id"), "doc_id should be omitted: {json}");
        assert!(
            !json.contains("chunk_index"),
            "chunk_index should be omitted: {json}"
        );
    }

    // ── error display ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_error_display() {
        assert_eq!(
            KnowledgeError::Backend("connection refused".to_string()).to_string(),
            "knowledge backend error: connection refused"
        );
        assert_eq!(
            KnowledgeError::InvalidTenant("bad!".to_string()).to_string(),
            "invalid tenant: bad!"
        );
        assert_eq!(
            KnowledgeError::NotConfigured.to_string(),
            "knowledge provider not configured"
        );
    }

    // ── object-safety ──────────────────────────────────────────────────────

    struct StubKnowledge;

    #[async_trait]
    impl Knowledge for StubKnowledge {
        async fn ingest(
            &self,
            _tenant: &TenantCtx,
            chunks: Vec<KnowledgeChunk>,
        ) -> KnowledgeResult<IngestOutcome> {
            Ok(IngestOutcome {
                chunk_ids: chunks.iter().map(|c| format!("id:{}", c.doc_id)).collect(),
            })
        }

        async fn search(
            &self,
            _tenant: &TenantCtx,
            query: KnowledgeQuery,
        ) -> KnowledgeResult<Vec<RetrievedChunk>> {
            Ok(vec![RetrievedChunk {
                text: format!("result for: {}", query.query),
                score: 1.0,
                doc_id: None,
                chunk_index: None,
                metadata: Map::new(),
            }])
        }
    }

    #[tokio::test]
    async fn trait_is_object_safe_and_both_methods_callable() {
        let kb: Box<dyn Knowledge> = Box::new(StubKnowledge);
        let t = tenant("tenant-a");

        let chunks = vec![KnowledgeChunk {
            doc_id: "doc-1".to_string(),
            chunk_index: 0,
            text: "Hello world.".to_string(),
            metadata: Map::new(),
            embedding: None,
        }];

        let outcome = kb.ingest(&t, chunks).await.expect("ingest succeeds");
        assert_eq!(outcome.chunk_ids, vec!["id:doc-1"]);

        let results = kb
            .search(
                &t,
                KnowledgeQuery {
                    query: "hello".to_string(),
                    limit: Some(3),
                },
            )
            .await
            .expect("search succeeds");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "result for: hello");
    }
}
