# greentic-dw-memory-long-term

Long-term episodic memory contract for Greentic DW — semantic, bi-temporal,
backend-agnostic.

This crate defines the `LongTermMemory` trait and the data-transfer objects
shared by every long-term memory backend. It deliberately contains no concrete
backend and no chronicle types; those live in sibling crates, so a consumer can
depend on the contract without pulling in graph infrastructure.

## What is here

- `LongTermMemory` — the async trait backends implement: ingest episodes,
  recall facts semantically.
- `EpisodeIngest` / `IngestOutcome` — the write side.
- `RecallQuery` / `RecalledFact` — the read side.
- `LongTermMemoryError` — backend, tenant-validation and not-configured
  failures.

## Why it is separate

The long-term tier is a different shape from the key-value working-memory seam
(`MemoryProvider` in `greentic-aw-runtime`): it ingests episodes and recalls
facts rather than getting and setting keys. Keeping the contract in its own
lightweight crate lets the agentic-worker runtime compile and run without a
graph store, while the concrete backend (Chronicle over a graph database) is
injected at the runner-host edge.

## License

See the repository root.
