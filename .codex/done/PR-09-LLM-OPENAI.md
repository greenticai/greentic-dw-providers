# PR-09: Add native OpenAI provider

## Objective
Add a first-party OpenAI provider crate that targets native OpenAI semantics rather than generic compatibility mode.

## Deliverables
- `llm/openai` crate
- native OpenAI transport and client integration
- provider declaration and pack metadata
- provider-specific config schema
- provider-specific wizard QA block
- tests for text, structured outputs, and tool-call passthrough

## Config Fields
- `api_key_secret`
- `base_url` optional
- `model`
- `organization` optional
- `project` optional
- `timeout_ms`
- `allow_tools`
- `allow_structured_outputs`
- `reasoning_profile` optional

## Review Notes
- Keep this crate distinct from generic OpenAI-compatible transports because Responses API statefulness and built-in tool semantics are materially richer.
- Expose first-party support for stateful chaining and built-in tools through the shared feature profile instead of burying it in prose docs.
