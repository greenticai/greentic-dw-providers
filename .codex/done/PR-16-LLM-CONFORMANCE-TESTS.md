# PR-16: Add LLM family conformance tests

## Objective
Prove the LLM family shares the same naming, manifest, capability, feature-profile, and config-schema conventions across providers.

## Deliverables
- `crates/greentic-dw-providers-common/tests/pr07.rs` LLM-family test module
- helper-level conformance checks
- provider-level config-schema coverage through the real backend crates

## Shared Assertions
- naming conventions
- pack manifest behavior
- capability ids
- feature profile presence
- config schema exposure

## Minimum Test Matrix
- plain text generation
- structured output request
- tool-call request
- unsupported-feature behavior
- model and config validation

## Review Notes
- Keep family conformance tests in the shared helper crate, but leave protocol fidelity tests inside each provider crate.
- The unsupported-feature path is part of the contract. It should be tested just as aggressively as the success path.

## Status
Complete.

The shared helper crate now includes `tests/pr07.rs`, which verifies:

- provider ids, manifests, declarations, and pack manifests stay aligned across the implemented LLM backends
- feature profiles match the normalized `llm/core` request-fixture matrix for text, structured output, tool-calling, multimodal, and stateful requests
- provider config-schema fixtures expose the shared family basics
- the implemented wizard QA registry matches the same provider set
