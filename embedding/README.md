# Embedding provider family (`dw.embedding`)

Capability `cap://dw.embedding` (pack id `greentic.cap.embedding`). Produces text
embeddings for the Greentic DW retrieval/RAG path. Mirrors the `llm` family: a
normalized contract crate (`greentic-dw-embedding`) plus per-backend crates.

| Backend | Crate | Notes |
| --- | --- | --- |
| `openai` | `greentic-dw-embedding-openai` | Native OpenAI `/embeddings`; https only. |
| `openai-compatible` | `greentic-dw-embedding-openai-compatible` | Any OpenAI-shaped endpoint via required `base_url`; http(s). |

Default model `text-embedding-3-small`, default dim `1024`. Vectors are
truncated (never padded) to `embedding_dim`. `api_key_secret` is a secret
**reference** resolved by the runtime host — never a bare key.

Consumed by the Chronicle doc-RAG adapter (W2) and the `cap://dw.knowledge`
family (W3). See `greentic-designer/docs/superpowers/specs/2026-06-16-dw-knowledge-rag-design.md`.
