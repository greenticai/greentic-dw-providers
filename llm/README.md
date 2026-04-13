# LLM Providers

This directory contains the LLM provider family.

The shared helper crate currently defines the family-level LLM contract metadata:

- `cap://dw.llm`
- `greentic.cap.llm`

The current contract split is:

- `llm/core/` for the shared normalized LLM contract crate
- `llm/anthropic/` for Anthropic Claude via the Messages API
- `llm/azure-openai/` for Azure OpenAI with Azure-specific deployment and auth handling
- `llm/bedrock/` for Amazon Bedrock via the Converse API family
- `llm/gemini/` for Gemini via the `generateContent` API
- `llm/openai/` for the native OpenAI Responses API backend
- `llm/openai-compatible/` for generic OpenAI-shaped gateways such as Ollama and vLLM
- `llm/nvidia-nim/` for NVIDIA NIM with discovery and health helpers

The provider helper crate now exposes:

- `LlmFeatureProfile`
- `llm_provider_id`
- `llm_capability_uri`
- `llm_capability_id`
- `llm_pack_capability_id`
- `llm_provider_manifest`
- `llm_provider_declaration`
- `llm_capability_declaration`
- `llm_pack_manifest`
- `llm_pack_manifest_cbor`

The `llm/core` crate now exposes the normalized provider contract:

- `LlmRequest`
- `LlmMessage`
- `LlmContentPart`
- `LlmToolSpec`
- `LlmToolChoice`
- `LlmStructuredOutputSpec`
- `LlmResponse`
- `LlmToolCall`
- `LlmUsage`
- `LlmProviderFeatures`
- `LlmProvider`
- conformance fixture helpers for plain text, structured output, tool-calling, multimodal, and stateful request shapes

Future provider crates under this family are expected to reuse those helpers so provider ids,
capability ids, pack manifests, and feature-flag metadata stay aligned.

The backend crates now also expose provider-owned wizard QA metadata using the shared helper
types so setup UIs can render dynamic follow-up questions without vendor-specific branching.

The shared helper crate also carries an LLM-family conformance suite that checks the current
providers against shared expectations for ids, manifests, feature profiles, wizard metadata,
and exported config schemas.
