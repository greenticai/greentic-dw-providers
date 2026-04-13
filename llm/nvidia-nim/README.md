# NVIDIA NIM LLM Provider

This crate implements the shared `llm/core` contract for NVIDIA NIM LLM
deployments.

It stays separate from `llm/openai-compatible` because NIM is more than a raw
OpenAI-shaped endpoint:

- OpenAI-compatible inference for text, chat, tools, and streaming
- optional NIM-aware mode for model discovery
- health probes for readiness and liveness

The first implementation stays focused on the LLM family only.
