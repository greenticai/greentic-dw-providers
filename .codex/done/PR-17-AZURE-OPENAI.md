# PR-17: Add Azure OpenAI provider

## Title
`feat(llm-azure-openai): add Azure OpenAI provider`

## Objective
Add a first-class Azure OpenAI backend under `dw-providers` that implements the shared `llm/core` contract while exposing Azure-specific configuration, auth, deployment, and capability metadata.

## Why This Should Be Its Own Provider
- Keep this separate from both `llm/openai` and `llm/openai-compatible`.
- Azure OpenAI now documents its own Responses API path, supports stateful responses, supports structured outputs in GA `v1`, and supports both API key auth and Microsoft Entra ID.
- Its endpoint shape and deployment model are Azure-specific, so treating it as “just OpenAI-compatible” would hide important operational and capability differences.

## Provider Scope
- normal text generation
- structured outputs
- tool calling / function calling
- optional streaming
- stateful Responses API mode where supported by the selected model and config

## New Crate
- `llm/azure-openai/`

## Deliverables

### 1. Provider Crate
- Create:
  - `llm/azure-openai/Cargo.toml`
  - `llm/azure-openai/src/lib.rs`
- Add:
  - provider config types
  - provider implementation of the shared `llm/core` trait
  - provider declaration and pack manifest helpers using `greentic-dw-providers-common`

### 2. Provider Config Model
- Add an Azure-specific config type with fields such as:
  - `endpoint`
    - example: `https://{resource-name}.openai.azure.com`
  - `deployment`
    - Azure uses deployed model names, so this should be first-class
  - `api_version`
    - default to `v1`
  - `auth_mode`
    - `api_key`
    - `entra_id`
  - `api_key_secret` optional
  - `entra_scope` optional
  - `timeout_ms`
  - `allow_tools`
  - `allow_structured_outputs`
  - `allow_stateful_responses`
  - `use_responses_api`
  - `raw_headers` optional
- Azure-specific endpoint, auth mode, and deployment fields should be explicit instead of hidden inside generic compatibility config.

### 3. Feature Profile Metadata
- Expose provider features through the shared LLM feature profile, for example:
  - `chat = true`
  - `structured_outputs = true`
  - `tool_calling = true`
  - `streaming = true`
  - `stateful_conversation = true`
  - `enterprise_auth = true`
  - `local_self_hosted = false`
- Reflect current Azure OpenAI capabilities for Responses, function calling, and structured outputs while still allowing model- or deployment-level validation to reject unsupported combinations.

### 4. Request Mapping
- Implement request normalization from `llm/core` into Azure OpenAI request shapes.
- Support:
  - plain generation
  - structured output requests
  - function/tool declarations
  - optional response retrieval and stateful response chaining when enabled
- Prefer the Azure OpenAI Responses API when `use_responses_api = true`, since Azure documents that API as the unified stateful response surface.

### 5. Error Mapping
- Normalize Azure-specific errors into the shared `llm/core` error model:
  - auth and config errors
  - unsupported feature or model errors
  - timeout and network errors
  - invalid schema and structured-output request failures

### 6. Provider Declaration And Pack Metadata
- Add helpers to `crates/greentic-dw-providers-common/src/llm.rs` for:
  - Azure provider ids
  - capability ids
  - pack capability ids
  - provider manifest generation
  - feature profile declaration
- Suggested identifiers:
  - capability contract: `cap://dw.llm`
  - provider id: `dw.llm.azure-openai`
  - pack capability id: `greentic.cap.llm.azure-openai`

### 7. Wizard QA Block
- Add Azure-specific wizard questions for dynamic provider follow-up:
  - Azure OpenAI endpoint
  - deployment name
  - auth method
  - API key secret reference if using API key
  - Entra ID enabled
  - use Responses API
  - enable structured outputs
  - enable tool calling
  - timeout
- This should stay provider-specific because Azure configuration differs materially from plain OpenAI API usage.

### 8. Tests
- Add:
  - unit tests for config validation
  - mapping tests for text generation
  - mapping tests for structured outputs
  - mapping tests for tool calling
  - provider declaration and manifest tests
  - feature profile tests
- Also add family-level conformance coverage in the shared test suite.

## Design Notes

### Keep This Separate From `openai-compatible`
- Even though Azure OpenAI is related to OpenAI models, it should not be folded into the generic compatibility provider because:
  - endpoint structure is Azure-specific
  - auth can be API key or Entra ID
  - deployment names are Azure-specific
  - feature availability can depend on Azure deployment and model support
  - Azure documents its own Responses API surface and versioning model

### Prefer Deployed Model Terminology
- The config should use `deployment` as the first-class identifier, not just `model`, because Azure users interact with deployed model names rather than only raw model identifiers.

### Structured Output Caveats
- Structured outputs are supported in Azure OpenAI, including in GA `v1`, but Azure documents limitations, including unsupported scenarios like some Assistants or Agents paths and some model or version combinations.
- The provider should reflect this through capability flags and validation, not assume universal support.

## Suggested Crate Layout
```text
llm/
  azure-openai/
    Cargo.toml
    src/
      lib.rs
      config.rs
      provider.rs
      mapping.rs
      auth.rs
      errors.rs
      qa.rs
```

## Acceptance Criteria
- `llm/azure-openai` exists and compiles
- it implements the shared `llm/core` provider contract
- it exposes Azure-specific config and auth modes
- it supports plain text requests
- it supports structured outputs
- it supports function and tool calling
- it publishes provider metadata through `greentic-dw-providers-common`
- it contributes a provider-driven QA block for the DW wizard
- tests cover config, request mapping, feature flags, and manifest and declaration output

## Status
Complete.

The repository now includes `llm/azure-openai` with Azure-specific endpoint, deployment, and auth
handling; Responses API and Chat Completions request mapping; shared helper metadata and wizard QA
integration; and shared-helper plus family-level conformance coverage.
