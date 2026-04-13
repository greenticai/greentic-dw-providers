# PR-14: Add Bedrock provider

## Objective
Add a Bedrock provider crate that treats Bedrock as a first-class backend surface through Converse and ConverseStream.

## Deliverables
- `llm/bedrock` crate
- Converse and ConverseStream mapping
- model-id-based backend selection
- tool-use mapping
- AWS auth config
- official AWS SDK transport wiring
- tests around request normalization

## Config Fields
- `region`
- `model_id`
- `auth_mode`
  - `env/default chain`
  - `profile`
  - access-key secret refs
- `assumed_role` later
- `timeout_ms`
- `allow_tools`

## Review Notes
- Keep this as one backend crate instead of model-vendor-specific Bedrock wrappers so enterprise users can standardize on one provider surface.
- The normalization tests matter more than mock completeness here because Bedrock support lives or dies on correct translation of the shared request model.
- The finished backend now uses the AWS SDK for Bedrock Runtime directly, including Converse and ConverseStream request execution behind the provider's sync transport abstraction.
