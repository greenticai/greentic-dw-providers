# OpenAI-Compatible LLM Provider

This crate implements the shared `llm/core` contract for gateways that expose
an OpenAI-shaped API without claiming full OpenAI parity.

It is intended for targets such as:

- Ollama
- vLLM
- LM Studio style gateways
- internal OpenAI-compatible gateways

Compatibility differences stay explicit through config:

- `compat_mode` selects `responses` or `chat_completions`
- `supports_stateful_responses` controls whether response chaining is exposed
- `supports_structured_outputs` controls whether JSON-schema output is exposed
