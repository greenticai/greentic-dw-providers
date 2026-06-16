# `cap://dw.knowledge` Provider Family (Knowledge/RAG epic — W3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans. Steps use `- [ ]`.

**Goal:** Add a `cap://dw.knowledge` provider family to `greentic-dw-providers` that wraps the Chronicle doc-RAG path (W2): ingest document chunks + hybrid retrieval, with a `DwEmbedderBridge` adapting the dw `EmbeddingProvider` (W1) to Chronicle's `EmbedderClient`. Mirrors the existing `memory/long-term/{core,chronicle}` family.

**Architecture:** `knowledge/core` defines a sync-free async `Knowledge` trait + DTOs + capability consts. `knowledge/chronicle` wraps `chronicle_core::Chronicle`, calling its W2 `ingest_document_chunks` / `search_document_chunks`; it is **driver-agnostic** (core constructor injects `Arc<dyn GraphDriver>`; the SurrealDB convenience constructor is behind an optional `surreal` feature so default/test builds don't need the RocksDB/bindgen toolchain). `DwEmbedderBridge` adapts dw `EmbeddingProvider` → chronicle `EmbedderClient` via `spawn_blocking` (mirrors the existing `DwLlmBridge`). `greentic-dw-providers-common` gains `ProviderCategory::Knowledge` + a `knowledge` helper module; the family registers in `unified_catalog` + `gtpacks.manifest.json`.

**Tech stack:** Rust edition 2024, workspace `1.2.0-research`, rust 1.94. Reuses `greentic-dw-embedding` (W1, now on research), `chronicle-core` (W2 doc-RAG, now on research at rev `629464b`), `chronicle-testkit` (FakeDriver/MockEmbedder/MockLlm).

**Epic spec:** greentic-designer `docs/superpowers/specs/2026-06-16-dw-knowledge-rag-design.md`. **Precedent to mirror (READ IT):** `memory/long-term/{core,chronicle}` + `crates/greentic-dw-providers-common/src/memory.rs` (the long-term helpers).

**Conventions:** English only; `#![forbid(unsafe_code)]` at crate roots; no `unwrap()/panic!()` in non-test code; Conventional Commits `feat(knowledge):`; NO Claude co-author trailer. Build/test the new crates + common + long-term; do NOT enable the `surreal` feature locally (bindgen/RocksDB — sandbox caveat).

**Verified facts:**
- chronicle research rev with doc-RAG = **`629464b`**. Workspace chronicle pins are at `Cargo.toml:84-87` (currently `tag = "1.2.0-research"`).
- chronicle `EmbedderClient` (async): `fn embedding_dim(&self)->usize; async fn create(&self,&str)->Result<Vec<f32>,EmbedderError>; async fn create_batch(&self,&[String])->Result<Vec<Vec<f32>>,EmbedderError>`.
- chronicle doc-RAG: `Chronicle::ingest_document_chunks(Vec<chronicle_core::DocumentChunk>, group_id:&str)->Result<Vec<String>,ChronicleError>`; `search_document_chunks(query:&str, group_ids:&[String], limit:usize)->Result<Vec<chronicle_core::DocumentChunkHit>,ChronicleError>`. `DocumentChunk{doc_id,chunk_index,text,metadata}`, `DocumentChunkHit{text,score,group_id,doc_id:Option,chunk_index:Option,metadata}`.
- dw `EmbeddingProvider` (W1, sync): `fn features(&self)->&EmbeddingProviderFeatures; fn embed(&self,&TenantCtx,EmbeddingRequest)->EmbeddingResult<EmbeddingResponse>`. `EmbeddingResponse{dim, vectors:Vec<Vec<f32>>, ..}`. `EmbeddingProviderFeatures` has NO dim field → bridge takes `embedding_dim` from config.
- `EmbeddingRequest::new(request_id, inputs:Vec<String>)` + `.with_model(..)`.
- chronicle's `EmbedderError` variants include a transport/`Other`-style constructor (confirm exact name in chronicle-core `embedder` module; use `EmbedderError::...` accordingly).
- `Chronicle::new(driver: Arc<dyn GraphDriver>, llm: Arc<dyn LlmClient>, embedder: Arc<dyn EmbedderClient>, max_concurrency)`.
- common helpers: `capability_uri(ProviderCategory, &str)`, `pack_capability_id(...)`, `provider_decl(ProviderDeclSpec{..})`; long-term mirrors in `common/src/memory.rs`.

---

## File Structure

### New
```
knowledge/core/Cargo.toml                 greentic-dw-knowledge
knowledge/core/src/lib.rs                  Knowledge trait + DTOs + (re-export caps)
knowledge/core/README.md
knowledge/chronicle/Cargo.toml             greentic-dw-knowledge-chronicle
knowledge/chronicle/src/lib.rs             KnowledgeChronicle (driver-agnostic) + Knowledge impl
knowledge/chronicle/src/bridge.rs          DwEmbedderBridge (EmbeddingProvider → EmbedderClient)
knowledge/chronicle/src/config.rs          KnowledgeConfig (+ Debug redaction)
knowledge/chronicle/README.md
crates/greentic-dw-providers-common/src/knowledge.rs   ProviderCategory::Knowledge helpers + KnowledgeVariant
```
### Modified
```
Cargo.toml                                 + 2 members; chronicle pins tag→rev 629464b; + chronicle-driver-surreal (optional); + greentic-dw-knowledge workspace dep
crates/greentic-dw-providers-common/src/category.rs   + ProviderCategory::Knowledge (+ as_str, FromStr, planned_categories N+1)
crates/greentic-dw-providers-common/src/lib.rs        + pub mod knowledge;
crates/greentic-dw-providers-common/src/catalog.rs    + knowledge field + entries + wiring
crates/greentic-dw-providers-common/tests/pr01.rs     + bump category count/list/banner (as W1 did for Embedding)
packs/gtpacks.manifest.json                + knowledge.chronicle entry
```

---

## Task 0: Worktree (DONE) + chronicle pin bump

Worktree: `~/Works/worktrees/dwp-knowledge` (branch `feat/dw-knowledge` off `origin/research`). Set:
```
cd ~/Works/worktrees/dwp-knowledge
export CARGO_TARGET_DIR=$HOME/.cache/cargo-target/dwp-knowledge
```

- [ ] **Step 1 — bump chronicle pins** (`Cargo.toml` lines ~84-87): change each `tag = "1.2.0-research"` to `rev = "629464b"` for `chronicle-core`, `chronicle-driver-neo4j`, `chronicle-llm-openai`, `chronicle-testkit`. ALSO add (for the optional surreal feature):
```toml
chronicle-driver-surreal = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", rev = "629464b" }
```
And add the new family core to `[workspace.dependencies]`:
```toml
greentic-dw-knowledge = { path = "knowledge/core" }
```
- [ ] **Step 2 — verify the bump doesn't break the existing workspace** (long-term/chronicle now builds against the newer chronicle rev):
```
cargo build -p greentic-dw-memory-chronicle
```
Expected: clean. If chronicle drifted since `1.2.0-research` and broke long-term, FIX the long-term call sites minimally (collateral; note it) — do NOT revert the bump.
- [ ] **Step 3 — commit:** `chore(knowledge): bump chronicle pin to research rev 629464b (doc-RAG API)`

---

## Task 1: `knowledge/core` — `Knowledge` trait + DTOs

Mirror `memory/long-term/core`. **Read** `memory/long-term/core/src/lib.rs` and `memory/long-term/core/Cargo.toml` as the template.

**Files:** `knowledge/core/Cargo.toml`, `knowledge/core/src/lib.rs`, `knowledge/core/README.md`; add `"knowledge/core"` to workspace members.

- [ ] **Step 1 — Cargo.toml** (mirror long-term/core; name `greentic-dw-knowledge`; deps: `async-trait`, `chrono`, `greentic-dw-providers-common`, `greentic-types`, `serde`, `serde_json`, `thiserror` — all `{ workspace = true }`).
- [ ] **Step 2 — failing test + lib.rs.** Implement:
```rust
#![forbid(unsafe_code)]
//! Normalized knowledge (document-RAG) provider contract.

use async_trait::async_trait;
use greentic_types::TenantCtx;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

pub use greentic_dw_providers_common::knowledge::{
    knowledge_capability_uri, knowledge_pack_capability_id,
};

/// One pre-chunked unit to ingest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub doc_id: String,
    pub chunk_index: usize,
    pub text: String,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

/// Outcome of an ingest call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IngestOutcome {
    pub chunk_ids: Vec<String>,
}

/// A retrieval query.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// A retrieval hit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RetrievedChunk {
    pub text: String,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_index: Option<usize>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

#[derive(Debug, Error)]
pub enum KnowledgeError {
    #[error("knowledge backend error: {0}")]
    Backend(String),
    #[error("invalid tenant: {0}")]
    InvalidTenant(String),
    #[error("knowledge provider not configured")]
    NotConfigured,
}

pub type KnowledgeResult<T> = Result<T, KnowledgeError>;

/// Contract implemented by knowledge backends. The knowledge `group_id` is
/// derived from the tenant by the provider (corpus isolation).
#[async_trait]
pub trait Knowledge: Send + Sync {
    async fn ingest(&self, tenant: &TenantCtx, chunks: Vec<KnowledgeChunk>) -> KnowledgeResult<IngestOutcome>;
    async fn search(&self, tenant: &TenantCtx, query: KnowledgeQuery) -> KnowledgeResult<Vec<RetrievedChunk>>;
}
```
Tests: DTO serde roundtrip; `KnowledgeError` Display; capability consts equal `cap://dw.knowledge` / `greentic.cap.knowledge` (these come from common — Task 2 must land first, OR stub the consts here and switch to the re-export after Task 2; to keep TDD order clean, do **Task 2 before Task 1's capability test**, or inline-assert the literal strings).
- [ ] **Step 3 — run** `cargo test -p greentic-dw-knowledge` (after Task 2 provides the common consts). Expected PASS.
- [ ] **Step 4 — commit:** `feat(knowledge): knowledge/core Knowledge trait + DTOs`

---

## Task 2: common — `ProviderCategory::Knowledge` + `knowledge` helper module

Mirror `crates/greentic-dw-providers-common/src/memory.rs` long-term helpers and the W1 `embedding.rs`/`category.rs` changes. **Read** `common/src/memory.rs` (the `long_term_memory_*` fns + `LongTermMemoryVariant`).

- [ ] **Step 1 — `category.rs`:** add `Knowledge` variant + `as_str => "knowledge"` + `FromStr` arm + bump `planned_categories()` array length (currently 9 after W1's Embedding → 10) appending `Knowledge`.
- [ ] **Step 2 — `tests/pr01.rs`:** bump the category count + expected array + banner string to include `knowledge` (exactly as W1 did for `embedding`).
- [ ] **Step 3 — create `common/src/knowledge.rs`** mirroring the long-term helpers:
```rust
//! Knowledge family metadata helpers.
use serde::{Deserialize, Serialize};
use crate::category::ProviderCategory;
// (mirror imports from memory.rs: provider_decl, ProviderDeclSpec, PackId, PackManifest, etc.)

#[must_use] pub fn knowledge_capability_uri() -> String { crate::capability_uri(ProviderCategory::Knowledge, "") .trim_end_matches('.').to_string() }
```
> NOTE: long-term uses `capability_uri(Memory, "long-term")` → `cap://dw.memory.long-term`. For knowledge the capability is the bare family `cap://dw.knowledge` (no sub-capability). Implement `knowledge_capability_uri()` to return exactly `"cap://dw.knowledge"` and `knowledge_pack_capability_id()` → `"greentic.cap.knowledge"` (construct directly as `format!("cap://dw.{}", ProviderCategory::Knowledge.as_str())` / `format!("greentic.cap.{}", ...)` to avoid a trailing dot). Add:
```rust
pub fn knowledge_operations() -> [&'static str; 2] { ["knowledge.ingest", "knowledge.search"] }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KnowledgeVariant { Chronicle }
impl KnowledgeVariant {
    pub const fn as_str(self) -> &'static str { match self { Self::Chronicle => "chronicle" } }
    pub fn component_ref(self) -> String { format!("component:knowledge.{}", self.as_str()) }
    pub fn provider_type(self) -> String { ProviderCategory::Knowledge.provider_type(self.as_str()) }
}
```
Plus `knowledge_provider_decl(variant)` + `knowledge_pack_capabilities(variant)` + `knowledge_pack_manifest(pack_id, variant)` — **mirror the `long_term_memory_*` equivalents exactly**, substituting category `Knowledge`, ops `knowledge_operations()`, schema refs `schemas/knowledge/{variant}.json`, component `component:knowledge.{variant}`, docs `docs/providers/knowledge/{variant}.md`, capability id `knowledge_pack_capability_id()`.
- [ ] **Step 4 — `lib.rs`:** `pub mod knowledge;`
- [ ] **Step 5 — tests** (in knowledge.rs): capability strings (`cap://dw.knowledge`, `greentic.cap.knowledge`); `KnowledgeVariant::Chronicle.provider_type()=="dw.knowledge.chronicle"`, `component_ref()=="component:knowledge.chronicle"`. Run `cargo test -p greentic-dw-providers-common knowledge` + fix any newly-exhaustive `match ProviderCategory` sites (search `ProviderCategory::Embedding`).
- [ ] **Step 6 — commit:** `feat(knowledge): ProviderCategory::Knowledge + common knowledge helpers`

---

## Task 3: catalog + pack manifest

- [ ] **Step 1 — `catalog.rs`:** add `knowledge: Vec<ProviderCatalogEntry>` to `ProviderCatalog`, `knowledge_entries()` (maps `[KnowledgeVariant::Chronicle]`), wire into `unified_catalog()`, add test `unified_catalog_includes_knowledge_family` (assert one entry `provider_type=="dw.knowledge.chronicle"`).
- [ ] **Step 2 — `packs/gtpacks.manifest.json`:** add
```json
{ "category": "knowledge", "name": "chronicle", "pack_id": "greentic.dw.providers.knowledge.chronicle" }
```
- [ ] **Step 3 — run** `cargo test -p greentic-dw-providers-common` (all). Expected PASS.
- [ ] **Step 4 — commit:** `feat(knowledge): register knowledge family in catalog + pack manifest`

---

## Task 4: `knowledge/chronicle` — config + `DwEmbedderBridge`

**Read** `memory/long-term/chronicle/src/{config.rs,bridge.rs,lib.rs}` as the template.

**Files:** `knowledge/chronicle/Cargo.toml`, `src/config.rs`, `src/bridge.rs`; add `"knowledge/chronicle"` to workspace members.

- [ ] **Step 1 — Cargo.toml.** Mirror long-term/chronicle; name `greentic-dw-knowledge-chronicle`; deps: `greentic-dw-knowledge`, `greentic-dw-embedding` (NEW — for `EmbeddingProvider`), `greentic-dw-llm`, `greentic-types`, `chronicle-core`, `chronicle-llm-openai`, `async-trait`, `chrono`, `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`; dev-dep `chronicle-testkit`. The neo4j driver and surreal driver are **optional**:
```toml
[dependencies]
chronicle-driver-neo4j = { workspace = true, optional = true }
chronicle-driver-surreal = { workspace = true, optional = true }
[features]
default = []
neo4j = ["dep:chronicle-driver-neo4j"]
surreal = ["dep:chronicle-driver-surreal"]
```
> Default build pulls NO real driver (no bindgen, no neo4rs) — tests use `chronicle-testkit::FakeDriver`. Real deployments enable `surreal` (or `neo4j`).
- [ ] **Step 2 — `config.rs`.** Mirror long-term's `ChronicleMemoryConfig` but knowledge-focused: `KnowledgeConfig { embedding_model: Option<String>, embedding_dim: usize, openai_api_key: Option<String>, openai_base_url: Option<String>, llm_api_key: Option<String>, llm_model: Option<String>, db_path: Option<String> (for surreal) / neo4j_* (for neo4j), max_concurrency: usize, search_limit: usize }`. Builder methods + `Debug` redacting all `*_api_key` + neo4j_password. `embedding_dim` is REQUIRED (no Option) — it feeds both the bridge and the index. Tests: builder defaults; Debug redaction hides keys.
- [ ] **Step 3 — `bridge.rs` — `DwEmbedderBridge`** (mirror `DwLlmBridge`; CORRECTED to chronicle's real `EmbedderClient` methods):
```rust
use std::sync::Arc;
use async_trait::async_trait;
use greentic_dw_embedding::{EmbeddingProvider, EmbeddingRequest};
use greentic_types::TenantCtx;
use chronicle_core::embedder::{EmbedderClient, EmbedderError};

/// Adapts a dw [`EmbeddingProvider`] (sync) to chronicle's async [`EmbedderClient`].
pub struct DwEmbedderBridge {
    provider: Arc<dyn EmbeddingProvider>,
    tenant: TenantCtx,
    dim: usize,
}

impl DwEmbedderBridge {
    pub fn new(provider: Arc<dyn EmbeddingProvider>, tenant: TenantCtx, dim: usize) -> Self {
        Self { provider, tenant, dim }
    }

    async fn embed_inputs(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, EmbedderError> {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        let provider = Arc::clone(&self.provider);
        let tenant = self.tenant.clone();
        let request = EmbeddingRequest::new("chronicle-doc-rag", inputs);
        let response = tokio::task::spawn_blocking(move || provider.embed(&tenant, request))
            .await
            .map_err(|e| EmbedderError::transport(format!("embedder bridge join error: {e}")))?
            .map_err(|e| EmbedderError::transport(format!("dw embedding provider error: {e}")))?;
        Ok(response.vectors)
    }
}

#[async_trait]
impl EmbedderClient for DwEmbedderBridge {
    fn embedding_dim(&self) -> usize {
        self.dim
    }
    async fn create(&self, input: &str) -> Result<Vec<f32>, EmbedderError> {
        let mut out = self.embed_inputs(vec![input.to_string()]).await?;
        out.pop().ok_or_else(|| EmbedderError::transport("empty embedding response".to_string()))
    }
    async fn create_batch(&self, inputs: &[String]) -> Result<Vec<Vec<f32>>, EmbedderError> {
        self.embed_inputs(inputs.to_vec()).await
    }
}
```
> CONFIRM the exact `EmbedderError` constructor name in chronicle-core (`EmbedderError::transport(..)` vs `EmbedderError::Transport(..)` vs `::provider(..)`) — read `chronicle-core/src/embedder/mod.rs` and use the real one. CONFIRM the `EmbedderClient` import path (`chronicle_core::embedder::EmbedderClient`).
- [ ] **Step 4 — tests** for the bridge using `MockEmbedder`? No — `DwEmbedderBridge` wraps a dw `EmbeddingProvider`, not chronicle's MockEmbedder. Use the W1 `greentic-dw-embedding-openai` provider with a mock transport, OR a tiny in-test `EmbeddingProvider` impl returning fixed vectors. Implement a fixture `StubEmbeddingProvider` in the test module that returns `dim`-length vectors; assert `create`/`create_batch`/`embedding_dim` map correctly (count, dim). Run `cargo test -p greentic-dw-knowledge-chronicle bridge`.
- [ ] **Step 5 — commit:** `feat(knowledge): chronicle config + DwEmbedderBridge (EmbeddingProvider→EmbedderClient)`

---

## Task 5: `knowledge/chronicle` — `KnowledgeChronicle` provider + `Knowledge` impl

**Files:** `knowledge/chronicle/src/lib.rs`.

- [ ] **Step 1 — driver-agnostic core + Knowledge impl:**
```rust
#![forbid(unsafe_code)]
mod bridge;
mod config;
pub use bridge::DwEmbedderBridge;
pub use config::KnowledgeConfig;

use std::sync::Arc;
use async_trait::async_trait;
use chronicle_core::chronicle::Chronicle;
use chronicle_core::driver::GraphDriver;
use chronicle_core::embedder::EmbedderClient;
use chronicle_core::llm::LlmClient;
use chronicle_core::{DocumentChunk as CDocumentChunk};
use greentic_dw_knowledge::{
    IngestOutcome, Knowledge, KnowledgeChunk, KnowledgeError, KnowledgeQuery, KnowledgeResult,
    RetrievedChunk,
};
use greentic_types::TenantCtx;

const DEFAULT_SEARCH_LIMIT: usize = 10;

/// Chronicle-backed knowledge provider (lite doc-RAG). Driver-agnostic: the
/// caller injects the graph driver (FakeDriver in tests; SurrealDB/Neo4j in
/// production behind crate features).
pub struct KnowledgeChronicle {
    chronicle: Chronicle,
    search_limit: usize,
}

impl KnowledgeChronicle {
    /// Build from fully-injected parts (the testable seam).
    pub async fn from_parts(
        driver: Arc<dyn GraphDriver>,
        llm: Arc<dyn LlmClient>,
        embedder: Arc<dyn EmbedderClient>,
        max_concurrency: usize,
        search_limit: usize,
    ) -> KnowledgeResult<Self> {
        let chronicle = Chronicle::new(driver, llm, embedder, max_concurrency);
        chronicle
            .build_indices_and_constraints(false)
            .await
            .map_err(|e| KnowledgeError::Backend(e.to_string()))?;
        Ok(Self { chronicle, search_limit: if search_limit == 0 { DEFAULT_SEARCH_LIMIT } else { search_limit } })
    }
}

/// Derive the knowledge corpus group_id from the tenant (corpus isolation).
fn group_id_for(tenant: &TenantCtx) -> String {
    format!("knowledge:{}", tenant.tenant_id()) // CONFIRM TenantCtx accessor (tenant_id()/as_str())
}

#[async_trait]
impl Knowledge for KnowledgeChronicle {
    async fn ingest(&self, tenant: &TenantCtx, chunks: Vec<KnowledgeChunk>) -> KnowledgeResult<IngestOutcome> {
        let group_id = group_id_for(tenant);
        let cchunks: Vec<CDocumentChunk> = chunks
            .into_iter()
            .map(|c| CDocumentChunk { doc_id: c.doc_id, chunk_index: c.chunk_index, text: c.text, metadata: c.metadata })
            .collect();
        let ids = self
            .chronicle
            .ingest_document_chunks(cchunks, &group_id)
            .await
            .map_err(|e| KnowledgeError::Backend(e.to_string()))?;
        Ok(IngestOutcome { chunk_ids: ids })
    }

    async fn search(&self, tenant: &TenantCtx, query: KnowledgeQuery) -> KnowledgeResult<Vec<RetrievedChunk>> {
        let group_id = group_id_for(tenant);
        let limit = query.limit.unwrap_or(self.search_limit);
        let hits = self
            .chronicle
            .search_document_chunks(&query.query, &[group_id], limit)
            .await
            .map_err(|e| KnowledgeError::Backend(e.to_string()))?;
        Ok(hits
            .into_iter()
            .map(|h| RetrievedChunk { text: h.text, score: h.score, doc_id: h.doc_id, chunk_index: h.chunk_index, metadata: h.metadata })
            .collect())
    }
}
```
> CONFIRM: `chronicle_core::DocumentChunk` field names/visibility (from W2: `doc_id, chunk_index, text, metadata` all pub); the `TenantCtx` accessor for the tenant id; import paths for `Chronicle`/`GraphDriver`/`EmbedderClient`/`LlmClient`.
- [ ] **Step 2 — integration tests** (`knowledge/chronicle/tests/` per testkit pattern; dev-dep `chronicle-testkit` + `greentic-dw-knowledge` + `tokio`): build `KnowledgeChronicle::from_parts(Arc::new(FakeDriver::new()), Arc::new(MockLlm::new(vec![])), Arc::new(MockEmbedder::new(4)), 0, 5)`, ingest 2 chunks → assert `chunk_ids.len()==2` + idempotent (re-ingest same ids); search empty query → empty; search a query → no error, hits are `RetrievedChunk` with mapped provenance. (Mirror the W2 testkit tests.)
- [ ] **Step 3 — run** `cargo test -p greentic-dw-knowledge-chronicle` (default features — no surreal/neo4j build). Expected PASS.
- [ ] **Step 4 — commit:** `feat(knowledge): KnowledgeChronicle provider + Knowledge impl over doc-RAG`

---

## Task 6: README + gate

- [ ] **Step 1 — READMEs** for `knowledge/core` + `knowledge/chronicle` (short: capability `cap://dw.knowledge`, ops, the embedder-bridge + driver-feature notes, link epic spec).
- [ ] **Step 2 — scoped gate:**
```
cargo test -p greentic-dw-knowledge -p greentic-dw-knowledge-chronicle -p greentic-dw-providers-common
cargo clippy -p greentic-dw-knowledge -p greentic-dw-knowledge-chronicle -p greentic-dw-providers-common --all-targets -- -D warnings
cargo fmt --all -- --check
cargo build -p greentic-dw-memory-chronicle    # confirm the chronicle pin bump didn't break long-term
```
All pass. Do NOT run `--all-features` / the `surreal` feature (bindgen). If `bash ci/local_check.sh` is attempted and fails ONLY on the surreal/RocksDB build, document it as the known sandbox caveat.
- [ ] **Step 3 — commit:** `feat(knowledge): READMEs + W3 done`
- [ ] **Step 4 — push (await controller)** — controller handles push + PR to `research`.

---

## Self-Review
- **Spec coverage:** Knowledge trait+DTOs (T1), capability/category/helpers (T2), catalog+pack (T3), config+bridge (T4), provider+impl (T5), docs+gate (T6). Pin bump (T0) unblocks the doc-RAG API.
- **Layering:** knowledge/chronicle depends on greentic-dw-embedding (W1) + chronicle-core (W2). Driver-agnostic core + feature-gated drivers → default build needs no bindgen. Embedder bridge mirrors the proven DwLlmBridge.
- **Confirm-in-code:** exact `EmbedderError` constructor + `EmbedderClient`/`Chronicle`/`GraphDriver`/`LlmClient` import paths; `TenantCtx` tenant-id accessor; `chronicle_core::DocumentChunk` re-export path + field visibility; the exhaustive `match ProviderCategory` sites needing a `Knowledge` arm; `planned_categories()` current length (9 after W1 → 10).
- **Follow-up (W3→W4):** validate `EmbeddingResponse.dim == config.embedding_dim` in the bridge/ingest (deferred W2 follow-up); real-backend (surreal/neo4j) retrieval test belongs to a feature-gated CI lane.
- **No publish risk:** chronicle pinned by REV (not a new tag) → no chronicle publish triggered.
