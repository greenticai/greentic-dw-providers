#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Long-term episodic memory family contract — semantic, bi-temporal; backend-agnostic.
//!
//! This crate defines the [`LongTermMemory`] trait and the data-transfer objects
//! shared by all long-term memory backends. No concrete backend or chronicle types
//! are present here; those live in sibling crates.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use greentic_types::TenantCtx;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Error ──────────────────────────────────────────────────────────────────

/// Errors returned by long-term memory operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LongTermMemoryError {
    /// The underlying storage or graph backend returned an error.
    #[error("backend error: {0}")]
    Backend(String),

    /// The tenant identifier is invalid or unknown.
    #[error("invalid tenant: {0}")]
    InvalidTenant(String),

    /// No long-term memory backend has been configured for this tenant.
    #[error("not configured: {0}")]
    NotConfigured(String),
}

// ─── DTOs ───────────────────────────────────────────────────────────────────

/// Provenance category of an ingested episode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeSource {
    /// A conversation message (chat turn, user/assistant utterance, etc.).
    Message,
    /// A plain-text document or passage.
    Text,
    /// A structured JSON payload.
    Json,
}

/// A single episode submitted for ingestion into long-term memory.
///
/// An episode is the atomic unit of knowledge: one conversation turn, one
/// document chunk, or one structured event. The backend is responsible for
/// extracting entities, facts, and relationships from [`body`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeIngest {
    /// Human-readable name for this episode (e.g. "turn-42", "doc:readme").
    pub name: String,
    /// Raw content of the episode — free-form text or serialised JSON.
    pub body: String,
    /// How the body should be interpreted during extraction.
    pub source: EpisodeSource,
    /// Optional human-readable description of the source system or channel.
    pub source_description: Option<String>,
    /// Wall-clock time the event described by this episode occurred.
    /// Used as the *valid-time* anchor for bi-temporal indexing.
    pub reference_time: DateTime<Utc>,
}

/// Result returned after a successful ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestOutcome {
    /// Backend-assigned identifier for the stored episode.
    pub episode_id: String,
    /// Number of facts (edges) extracted and persisted.
    pub fact_count: usize,
    /// Number of entity nodes created or updated.
    pub entity_count: usize,
}

/// Parameters for a semantic recall operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallQuery {
    /// Natural-language question or keyword phrase.
    pub query: String,
    /// Maximum number of facts to return. `None` lets the backend decide.
    pub limit: Option<usize>,
}

/// A single fact retrieved from long-term memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalledFact {
    /// Human-readable statement of the fact.
    pub fact: String,
    /// Relationship type linking the subject and object (e.g. `"knows"`, `"prefers"`).
    pub relation: String,
    /// Valid-time start: when this fact became true. `None` if unknown.
    pub valid_at: Option<DateTime<Utc>>,
    /// Valid-time end: when this fact ceased to be true. `None` means still current.
    pub invalid_at: Option<DateTime<Utc>>,
    /// Episode IDs from which this fact was derived.
    pub source_episode_ids: Vec<String>,
}

// ─── Trait ──────────────────────────────────────────────────────────────────

/// Contract for long-term episodic memory backends.
///
/// Implementations are expected to:
/// * extract entities and facts from each [`EpisodeIngest`] during [`ingest_episode`];
/// * support bi-temporal indexing so that [`recall`] can reason over the state of
///   knowledge at any point in time;
/// * scope all storage and retrieval strictly to the provided [`TenantCtx`].
///
/// No specific graph or vector technology is mandated here — backends may use
/// Neo4j, an embedded graph, a vector store, or any combination.
#[async_trait]
pub trait LongTermMemory: Send + Sync {
    /// Ingest one episode (conversation turn, document, event) into long-term memory.
    ///
    /// The backend extracts entities and facts from `episode.body`, merges them
    /// with existing knowledge, and returns an [`IngestOutcome`] summarising the
    /// changes.
    async fn ingest_episode(
        &self,
        tenant: &TenantCtx,
        episode: EpisodeIngest,
    ) -> Result<IngestOutcome, LongTermMemoryError>;

    /// Semantic recall: natural-language query over remembered facts.
    ///
    /// Returns a ranked list of [`RecalledFact`] entries that are relevant to
    /// `query`, ordered by relevance descending. The result set is bounded by
    /// `query.limit` when provided.
    async fn recall(
        &self,
        tenant: &TenantCtx,
        query: RecallQuery,
    ) -> Result<Vec<RecalledFact>, LongTermMemoryError>;
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use greentic_types::{EnvId, TenantId};

    // ── helpers ────────────────────────────────────────────────────────────

    fn tenant(value: &str) -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from(value).expect("tenant id"),
        )
    }

    fn fixed_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    // ── stub impl for object-safety test ──────────────────────────────────

    struct StubLongTermMemory;

    #[async_trait]
    impl LongTermMemory for StubLongTermMemory {
        async fn ingest_episode(
            &self,
            _tenant: &TenantCtx,
            episode: EpisodeIngest,
        ) -> Result<IngestOutcome, LongTermMemoryError> {
            Ok(IngestOutcome {
                episode_id: format!("stub-{}", episode.name),
                fact_count: 1,
                entity_count: 1,
            })
        }

        async fn recall(
            &self,
            _tenant: &TenantCtx,
            _query: RecallQuery,
        ) -> Result<Vec<RecalledFact>, LongTermMemoryError> {
            Ok(vec![RecalledFact {
                fact: "stub fact".into(),
                relation: "related_to".into(),
                valid_at: None,
                invalid_at: None,
                source_episode_ids: vec!["stub-ep-1".into()],
            }])
        }
    }

    // ── object-safety ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn trait_is_object_safe_and_both_methods_callable() {
        let memory: Box<dyn LongTermMemory> = Box::new(StubLongTermMemory);
        let t = tenant("tenant-a");

        let episode = EpisodeIngest {
            name: "turn-1".into(),
            body: "Alice met Bob".into(),
            source: EpisodeSource::Message,
            source_description: None,
            reference_time: fixed_ts(),
        };

        let outcome = memory
            .ingest_episode(&t, episode)
            .await
            .expect("ingest succeeds");
        assert_eq!(outcome.episode_id, "stub-turn-1");
        assert_eq!(outcome.fact_count, 1);

        let facts = memory
            .recall(
                &t,
                RecallQuery {
                    query: "Alice".into(),
                    limit: Some(5),
                },
            )
            .await
            .expect("recall succeeds");
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].fact, "stub fact");
    }

    // ── serde roundtrips ───────────────────────────────────────────────────

    #[test]
    fn episode_ingest_roundtrips_with_description() {
        let episode = EpisodeIngest {
            name: "doc-readme".into(),
            body: r#"{"key":"value"}"#.into(),
            source: EpisodeSource::Json,
            source_description: Some("internal docs system".into()),
            reference_time: fixed_ts(),
        };

        let json = serde_json::to_string(&episode).expect("serialize");
        let back: EpisodeIngest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.name, episode.name);
        assert_eq!(back.body, episode.body);
        assert_eq!(back.source, EpisodeSource::Json);
        assert_eq!(back.source_description, episode.source_description);
        assert_eq!(back.reference_time, episode.reference_time);
    }

    #[test]
    fn recalled_fact_roundtrips_with_temporal_fields_some() {
        let fact = RecalledFact {
            fact: "Alice knows Bob".into(),
            relation: "knows".into(),
            valid_at: Some(fixed_ts()),
            invalid_at: Some(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
                    .single()
                    .expect("ts"),
            ),
            source_episode_ids: vec!["ep-001".into(), "ep-002".into()],
        };

        let json = serde_json::to_string(&fact).expect("serialize");
        let back: RecalledFact = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.fact, fact.fact);
        assert_eq!(back.relation, fact.relation);
        assert_eq!(back.valid_at, fact.valid_at);
        assert_eq!(back.invalid_at, fact.invalid_at);
        assert_eq!(back.source_episode_ids, fact.source_episode_ids);
    }

    #[test]
    fn recalled_fact_roundtrips_with_temporal_fields_none() {
        let fact = RecalledFact {
            fact: "Alice likes Rust".into(),
            relation: "likes".into(),
            valid_at: None,
            invalid_at: None,
            source_episode_ids: vec![],
        };

        let json = serde_json::to_string(&fact).expect("serialize");
        let back: RecalledFact = serde_json::from_str(&json).expect("deserialize");
        assert!(back.valid_at.is_none());
        assert!(back.invalid_at.is_none());
        assert!(back.source_episode_ids.is_empty());
    }

    #[test]
    fn episode_source_serialises_lowercase() {
        assert_eq!(
            serde_json::to_string(&EpisodeSource::Message).expect("ser"),
            r#""message""#
        );
        assert_eq!(
            serde_json::to_string(&EpisodeSource::Text).expect("ser"),
            r#""text""#
        );
        assert_eq!(
            serde_json::to_string(&EpisodeSource::Json).expect("ser"),
            r#""json""#
        );
    }

    #[test]
    fn episode_source_deserialises_from_lowercase() {
        let msg: EpisodeSource = serde_json::from_str(r#""message""#).expect("deserialize message");
        assert_eq!(msg, EpisodeSource::Message);

        let txt: EpisodeSource = serde_json::from_str(r#""text""#).expect("deserialize text");
        assert_eq!(txt, EpisodeSource::Text);

        let jsn: EpisodeSource = serde_json::from_str(r#""json""#).expect("deserialize json");
        assert_eq!(jsn, EpisodeSource::Json);
    }
}
