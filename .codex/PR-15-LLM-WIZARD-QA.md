# PR-15: Add wizard QA descriptors for the LLM family

## Objective
Add provider-owned wizard metadata so the DW wizard can ask LLM-provider-specific follow-up questions dynamically.

## Deliverables
- provider QA blocks for:
  - OpenAI
  - OpenAI-compatible
  - Anthropic
  - Gemini
  - Bedrock
- shared wizard descriptor structs in the common helper crate
- i18n-ready prompt keys
- compatibility and default-selection metadata

## Shared Follow-Up Questions
- provider
- model
- timeout
- tool calling enabled
- structured outputs enabled

## Provider-Specific Follow-Up Questions
- OpenAI-compatible:
  - base URL
  - auth token optional
  - compatibility mode
- Bedrock:
  - region
  - auth method
  - model id
- Anthropic:
  - max tokens
  - thinking enabled
- Gemini:
  - API key source
  - model
  - structured-output preference

## Review Notes
- Keep wizard metadata provider-owned, but shaped by shared helper structs so the wizard does not need vendor-specific branching logic.
- Include visibility rules and defaults now; those are the pieces most likely to drift if each provider invents its own QA format later.
- The current implementation lands the shared helper structs plus provider-owned QA blocks with typed question kinds, defaults, compatibility tags, and simple visibility rules.
