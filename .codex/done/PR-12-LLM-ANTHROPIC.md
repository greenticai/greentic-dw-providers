# PR-12: Add Anthropic Claude provider

## Objective
Add an Anthropic provider crate that maps the Messages API, tool use, and structured outputs into the normalized LLM contract.

## Deliverables
- `llm/anthropic` crate
- Messages API integration
- tool-use mapping
- structured-output mapping
- provider QA and config
- tests for text, tools, and structured output

## Config Fields
- `api_key_secret`
- `model`
- `max_tokens`
- `timeout_ms`
- `allow_tools`
- `allow_thinking` optional
- `base_url` optional for proxying

## Review Notes
- Keep Anthropic’s content blocks and tool-use semantics normalized in the mapping layer rather than leaking them into `llm/core`.
- Make “thinking” an explicit optional capability/config path because not every deployment or policy will allow it.

## Implementation Status
- Complete.
- `llm/anthropic` now exists as a first-class provider crate.
- The provider currently includes:
  - Anthropic config validation and schema fixtures
  - Messages API request and response mapping
  - normalized tool-use mapping
  - structured output support via synthetic tool forcing in the mapping layer
  - explicit thinking-mode config and payload support
  - Anthropic-specific API error mapping
  - provider-owned wizard QA metadata
  - blocking `reqwest` transport
  - shared provider metadata wrappers in `greentic-dw-providers-common`
  - package tests for text, tools, structured outputs, feature flags, and metadata
