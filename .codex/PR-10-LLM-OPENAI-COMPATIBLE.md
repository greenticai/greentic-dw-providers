# PR-10: Add generic OpenAI-compatible provider

## Objective
Add a configurable OpenAI-compatible provider for self-hosted and gateway targets that speak an OpenAI-shaped API without claiming full OpenAI parity.

## Deliverables
- `llm/openai-compatible` crate
- configurable base URL and bearer-token auth
- compatibility mode flags
- tests using Ollama-shaped fixtures

## Supported Targets
- Ollama
- vLLM
- LM Studio style gateways
- custom internal OpenAI-compatible gateways

## Config Fields
- `base_url`
- `api_key_secret` optional
- `model`
- `headers` optional
- `compat_mode`
  - `responses`
  - `chat_completions`
- `supports_stateful_responses`
- `supports_structured_outputs`

## Review Notes
- Keep this provider separate from native OpenAI so compatibility gaps stay explicit in config and feature metadata.
- The contract should let this crate advertise non-stateful Responses support, which is the practical shape for Ollama-class backends today.
