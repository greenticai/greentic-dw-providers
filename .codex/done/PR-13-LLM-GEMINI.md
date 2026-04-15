# PR-13: Add Gemini provider

## Objective
Add a Gemini provider crate that maps `generateContent`, function calling, and structured outputs into the shared LLM contract.

## Delivered
- `llm/gemini` crate
- `generateContent` REST integration
- function-calling mapping through `tools.functionDeclarations` and `toolConfig.functionCallingConfig`
- structured-output mapping through `generationConfig.responseMimeType` and `responseJsonSchema`
- provider QA and config fixtures
- tests for text, tools, structured output, helper metadata, and request-shape mapping

## Config Fields
- `api_key_secret`
- `base_url`
- `model`
- `timeout_ms`
- `allow_tools`
- `allow_structured_outputs`
- `safety_profile`

## Notes
- Vertex-specific concerns remain out of scope for this crate and should land as a separate transport/auth path if needed.
- The current Gemini backend does not advertise multimodal input support yet because the normalized request currently carries remote image URLs, while the Gemini REST API expects inline or uploaded file data rather than a plain image URL contract.
