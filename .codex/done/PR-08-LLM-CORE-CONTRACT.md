# PR-08: Add normalized LLM core contract

## Objective
Define a provider-neutral LLM contract that normalizes the common request, response, tool, and structured-output surfaces without hardcoding one vendor API.

## Deliverables
- `llm/core/Cargo.toml`
- normalized request and response types
- provider feature-flag model
- trait definition
- error model
- conformance fixtures

## Core Types
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

## Design Constraints
- Do not encode OpenAI-native semantics as the core abstraction.
- Support stateful response chaining as an optional provider feature, not as a required conversation primitive.
- Separate “tool request,” “tool result,” and “structured output target” concepts so Anthropic, Gemini, Bedrock, and OpenAI can all map cleanly.
- Keep transport and auth concerns out of the core crate.

## Review Notes
- Treat this crate as the normalization boundary. Once it leaks vendor-specific semantics, every later provider crate gets harder to implement and test.
- Conformance fixtures should cover both supported and unsupported feature behavior so provider crates can fail predictably when a request exceeds their profile.
