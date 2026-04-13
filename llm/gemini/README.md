# Gemini Provider

This crate provides a Gemini backend for the shared normalized `llm/core` contract.

The first pass targets the REST `generateContent` API with:

- text generation
- function-calling
- JSON structured outputs via `responseMimeType` and `responseJsonSchema`
- provider-owned config and wizard metadata

Vertex-specific transport and auth concerns are intentionally left out of this crate.
