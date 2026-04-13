# Bedrock Provider

This crate provides an Amazon Bedrock backend for the shared normalized `llm/core` contract.

The first pass targets:

- `Converse` for standard text and tool-use requests
- `ConverseStream` for text-oriented streaming requests
- AWS credential configuration through the default chain, named profiles, or static credentials

It intentionally standardizes Bedrock as a single enterprise backend instead of splitting by model vendor.
