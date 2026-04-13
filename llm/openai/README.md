# OpenAI LLM Provider

This crate implements the shared `llm/core` contract against OpenAI's native
Responses API.

It is intentionally separate from generic OpenAI-compatible transports because
it supports OpenAI-specific behavior such as:

- `previous_response_id` stateful chaining
- JSON schema structured outputs
- first-party function tool calling
- OpenAI organization and project headers

The crate exposes:

- `OpenAiConfig`
- `OpenAiProvider`
- `OpenAiReasoningProfile`
- `openai_wizard_questions`

The canonical provider metadata stays aligned with the shared helper crate:

- provider id: `dw.llm.openai`
- runtime capability: `cap://dw.llm`
- pack capability: `greentic.cap.llm`
