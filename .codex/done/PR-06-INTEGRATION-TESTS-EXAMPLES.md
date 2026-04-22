# PR-06: Add integration tests and full examples

## Objective
Prove the provider ecosystem works with greentic-dw v0.2 and greentic-cap.

## Scope
- end-to-end example:
  - DW requiring short-term memory + task-state + audit
  - bundle resolves providers
  - runtime uses bindings
- examples with both:
  - in-memory providers
  - redis providers
- docs for OSS vs enterprise extension strategy

## Deliverables
- integration fixtures
- example bundle metadata
- example setup-time binding overrides
- polished docs

## Current Result
- The shared helper crate now exposes `EndToEndVariant`, `EndToEndExample`, `ExampleBundleMetadata`, `BundleResolutionFixture`, and `BindingOverride` helpers for the OSS and enterprise end-to-end scenarios.
- The repository also includes `examples/pr06/` with bundle, setup, and bundle-resolution JSON fixtures for both in-memory/OSS and redis/enterprise variants.
- The example fixtures cover short-term memory, task-state, and audit observer contracts together, which gives future bundle/setup work a single reference scenario.
