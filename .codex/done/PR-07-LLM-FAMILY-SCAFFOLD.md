# PR-07: Add LLM family scaffold and shared helpers

## Objective
Introduce the top-level `llm/` provider family and extend the shared helper crate so future LLM backends follow one naming and manifest pattern.

## Scope
- add the top-level `llm/` category and README scaffold
- add `llm/core` as the shared contract anchor crate
- extend `greentic-dw-providers-common` with LLM-specific helpers
- update repository docs, coverage policy, and perf fixtures if the new helper path affects them

## Shared Helper Deliverables
- `src/llm.rs`
- provider naming helpers:
  - `llm_provider_id("openai")`
  - display and variant naming helpers aligned with the other provider families
- capability helpers:
  - `llm_capability_id()`
  - `llm_pack_capability_id()`
  - capability/profile builders for family-level declarations
- provider metadata helpers:
  - `llm_provider_manifest(...)`
  - `llm_provider_declaration(...)`
  - `llm_feature_profile(...)`
- fixture builders for tests, examples, and future wizard integration

## Contract Metadata
- Each LLM provider profile should expose normalized support flags for:
  - `chat`
  - `structured_outputs`
  - `tool_calling`
  - `streaming`
  - `multimodal_input`
  - `stateful_conversation`
  - `local_self_hosted`
  - `enterprise_auth`
- Those flags should live in shared helper metadata so provider crates can declare capability differences explicitly instead of implying parity.

## Review Notes
- Keep the family capability broad at the family level, then put backend-specific variance into feature profiles rather than capability URI proliferation.
- Model OpenAI-compatible behavior as feature metadata, not as “partial OpenAI,” so Ollama-style differences stay explicit from day one.
- Put wizard-facing metadata in helper structs now even if the dynamic wizard integration lands later. That avoids having each provider crate invent its own ad hoc shape.
