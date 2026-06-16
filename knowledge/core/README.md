# greentic-dw-knowledge

Shared knowledge (document-RAG) provider contract for Greentic DW.

## Capability

- URI: `cap://dw.knowledge`
- Pack capability ID: `greentic.cap.knowledge`
- Operations: `knowledge.ingest`, `knowledge.search`

## Overview

This crate defines the `Knowledge` async trait and the normalized data-transfer
objects shared across all knowledge backends. No concrete backend or database
code lives here — those reside in sibling crates (`greentic-dw-knowledge-chronicle`).

### Key types

| Type | Purpose |
|---|---|
| `Knowledge` | Async trait: `ingest` + `search` |
| `KnowledgeChunk` | Pre-chunked unit to ingest (`doc_id`, `chunk_index`, `text`, `metadata`) |
| `IngestOutcome` | Backend-assigned chunk identifiers |
| `KnowledgeQuery` | Natural-language query + optional limit |
| `RetrievedChunk` | Retrieval hit: `text`, `score`, optional provenance |
| `KnowledgeError` | `Backend` / `InvalidTenant` / `NotConfigured` |

## Design

Mirrors the `greentic-dw-memory-long-term` pattern: a sync-free async trait with
a single async method pair so backends can be swapped without changing call sites.
All corpus isolation (group scoping per tenant) is enforced inside the backend
implementation.

## Epic spec

`greentic-designer/docs/superpowers/specs/2026-06-16-dw-knowledge-rag-design.md`
