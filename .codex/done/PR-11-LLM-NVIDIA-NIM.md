# PR-11: Add NVIDIA NIM provider

## Title
`feat(llm-nvidia-nim): add NVIDIA NIM provider`

## Objective
Add a first-class NVIDIA NIM backend under `dw-providers` that implements the shared `llm/core` contract while supporting both:
- OpenAI-compatible inference mode for chat, text, and streaming
- NIM-aware mode for model discovery, health, and future capability-aware routing

## Why This Should Be Its Own Provider
- Keep `nvidia-nim` separate from `openai-compatible`.
- NIM for LLMs exposes an OpenAI-compatible inference API and also has NIM-specific management endpoints.
- In the broader NeMo microservices stack, NVIDIA documents additional inference surfaces like embeddings and classification, and NIM Proxy can list available models via `/v1/models`.
- That makes NIM more than “just another OpenAI clone,” especially when model discovery, enterprise deployment awareness, and future expansion matter.

## Provider Scope
- Design this provider so a DW can use NIM today for LLM calls while leaving room for later NVIDIA-specific families such as embeddings, reranking, VLM, speech, or visual generation.
- Keep this PR focused on the LLM provider family only.

## New Crate
- `llm/nvidia-nim/`

## Deliverables

### 1. Provider Crate
- Create:
  - `llm/nvidia-nim/Cargo.toml`
  - `llm/nvidia-nim/src/lib.rs`
- Add:
  - provider config types
  - provider implementation of the shared `llm/core` trait
  - provider declaration and pack manifest helpers using `greentic-dw-providers-common`

### 2. Provider Config Model
- Add an NVIDIA-specific config type such as:
  - `base_url`
  - `api_mode`
    - `openai_compatible`
    - `nim_aware`
  - `model`
  - `api_key_secret` optional
  - `timeout_ms`
  - `allow_tools`
  - `allow_structured_outputs`
  - `allow_streaming`
  - `discover_models_on_start`
  - `healthcheck_on_start`
  - `headers` optional
  - `features_override` optional
- These fields matter because NIM exposes OpenAI-compatible inference endpoints, but the provider should still know it is targeting NIM and not a generic compatible gateway.

### 3. Feature Profile Metadata
- Expose provider features through the shared LLM feature profile, for example:
  - `chat = true`
  - `structured_outputs = configurable`
  - `tool_calling = configurable`
  - `streaming = true`
  - `stateful_conversation = false`
  - `enterprise_auth = optional`
  - `local_self_hosted = true`
  - `model_discovery = true`
- Model-specific support for structured outputs or tool calling should be expressed through capability flags plus config validation, not assumed universally.

### 4. Request Mapping
- Implement request normalization from `llm/core` into NIM request shapes.
- Support:
  - plain text generation
  - chat-style generation
  - optional structured output requests where compatible
  - optional tool and function declarations where compatible
  - streaming responses
- Default request execution to the OpenAI-compatible inference API, since that is the main documented inference surface for NIM LLMs.

### 5. NIM-Aware Discovery Helpers
- Add optional helpers for:
  - `list_models`
  - `health_ready`
  - `health_live`
- These should not be required for the basic `llm/core` contract, but they should exist behind provider-specific methods or internal helpers so the future DW wizard can:
  - validate connectivity
  - offer model choices dynamically
  - detect whether the target is really NIM and not just a generic compatible gateway

### 6. Error Mapping
- Normalize NVIDIA-specific failures into the shared `llm/core` error model:
  - base URL and config errors
  - auth errors
  - model not found
  - unsupported feature or model errors
  - timeout and network errors
  - invalid request and schema failures
- Also distinguish between:
  - “OpenAI-compatible request failed”
  - “NIM discovery or health endpoint failed”

### 7. Provider Declaration And Pack Metadata
- Add helpers to `crates/greentic-dw-providers-common/src/llm.rs` for:
  - NVIDIA NIM provider ids
  - capability ids
  - pack capability ids
  - provider manifest generation
  - feature profile declaration
- Suggested identifiers:
  - capability contract: `cap://dw.llm`
  - provider id: `dw.llm.nvidia-nim`
  - pack capability id: `greentic.cap.llm.nvidia-nim`

### 8. Wizard QA Block
- Add NVIDIA-specific wizard questions for dynamic provider follow-up:
  - NIM base URL
  - inference mode: standard compatible vs NIM-aware
  - model name
  - API key required
  - discover models automatically
  - run health check automatically
  - enable tool calling
  - enable structured outputs
  - enable streaming
  - timeout
- This keeps NIM represented as a real platform provider instead of a raw URL textbox.

### 9. Tests
- Add:
  - unit tests for config validation
  - mapping tests for chat and text requests
  - mapping tests for streaming config
  - model discovery parsing tests
  - health endpoint parsing tests
  - provider declaration and manifest tests
  - feature profile tests
- Also add family-level conformance coverage in the shared test suite.

## Design Notes

### Keep This Separate From `openai-compatible`
- The generic `openai-compatible` provider should remain the low-friction adapter for Ollama, vLLM-style gateways, and simple compatible endpoints.
- `nvidia-nim` should exist because:
  - NIM has documented management and discovery surfaces beyond basic inference
  - NIM is a platform and runtime product, not just a protocol facade
  - NVIDIA-aware behavior will likely matter later for model discovery and capability selection

### Start With LLM Scope Only
- Do not try to cram embeddings, reranking, VLM, speech, or image generation into this PR.
- NVIDIA has separate NIM surfaces across those modalities. This roadmap item should stay focused on the LLM provider family only.

### Treat Model Capabilities As Dynamic
- Do not hardcode assumptions like “all NIM models support tools” or “all NIM models support structured outputs.”
- Different models and deployments vary, so the provider should expose:
  - declared default feature flags
  - optional runtime detection
  - manual overrides

## Suggested Crate Layout
```text
llm/
  nvidia-nim/
    Cargo.toml
    src/
      lib.rs
      config.rs
      provider.rs
      mapping.rs
      discovery.rs
      errors.rs
      qa.rs
```

## Acceptance Criteria
- `llm/nvidia-nim` exists and compiles
- it implements the shared `llm/core` provider contract
- it supports basic NIM LLM inference through the OpenAI-compatible API
- it supports optional streaming
- it supports optional model discovery via `/v1/models`
- it supports optional health checks
- it publishes provider metadata through `greentic-dw-providers-common`
- it contributes a provider-driven QA block for the DW wizard
- tests cover config, inference mapping, discovery, health, feature flags, and manifest and declaration output

## Implementation Status
- Landed.
- The crate now enforces `api_mode` semantics:
  - `openai_compatible` is inference-only
  - `nim_aware` enables discovery and health helpers plus startup probing
- The provider exposes:
  - OpenAI-compatible LLM inference
  - `/v1/models` parsing
  - ready/live health parsing
  - provider-specific error mapping for inference vs discovery vs health failures
  - provider-owned wizard questions for NIM platform setup
  - startup probing driven by `discover_models_on_start` and `healthcheck_on_start`

## Suggested Follow-Up
- After this PR, the LLM family should be able to distinguish between:
  - pure protocol adapters like `openai-compatible`
  - platform-aware providers like `bedrock`, `azure-openai`, and `nvidia-nim`
