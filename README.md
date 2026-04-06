# greentic-dw-providers

Greentic digital worker provider workspace scaffold.

This repository is the starting point for provider implementations that will plug into the Greentic digital worker ecosystem. The intended provider families are:

- memory
- state
- engine
- control
- observer
- tool

The first shared code lives in [`crates/greentic-dw-providers-common`](crates/greentic-dw-providers-common), which reuses the shared Greentic crates instead of redefining core models locally:

- `greentic-types`
- `greentic-interfaces`
- `greentic-pack`
- `greentic-state`
- `greentic-cap-types` for capability declarations and pack capability patterns

The shared crate currently provides reusable helpers for:

- provider categories and naming
- capability declarations and pack capability ids
- provider manifests and runtime refs
- engine selection helpers
- short-term memory fixtures
- task-store fixtures
- engine fixtures
- control fixtures
- observer fixtures
- tool fixtures
- control bundle-resolution examples
- observer bundle-resolution examples
- tool bundle-resolution examples
- PR-06 end-to-end bundle/setup fixtures for OSS and enterprise strategies

## Category Conventions

Provider families follow a compact naming scheme that the shared helper crate enforces:

- provider types use `dw.<category>.<provider-name>`
- runtime capability URIs use `cap://dw.<category>.<capability>`
- pack capability ids use `greentic.cap.<category>.<capability>`

The common crate exposes a small builder surface for these patterns:

- `ProviderCategory`
- `ProviderDeclSpec` and `provider_decl`
- `provider_manifest`
- `sample_provider_decl`
- `sample_pack_manifest` and `sample_pack_manifest_cbor`
- `sample_capability_declaration_fixture`

These helpers are meant to be copied or wrapped by future provider crates so that manifests, pack extensions, and capability declarations stay aligned.

## Layout

The repository is organized by provider family:

- `engine/`
  - `default/`
  - `router-lite/`
- `control/`
  - `basic-policy/`
  - `delegation-guard/`
- `observer/`
  - `basic-audit/`
  - `basic-metrics/`
- `tool/`
  - `wasm-adapter/`
  - `mcp-adapter/`
- `memory/`
  - `short-term/in-memory/`
  - `short-term/redis/`
- `state/`
  - `task-store/in-memory/`
  - `task-store/redis/`

The category roots are still placeholders, but `engine/default/...`, `engine/router-lite/...`, `control/basic-policy/...`, `control/delegation-guard/...`, `observer/basic-audit/...`, `observer/basic-metrics/...`, `tool/wasm-adapter/...`, `tool/mcp-adapter/...`, `memory/short-term/...`, and `state/task-store/...` now carry concrete contract documentation for the first engine, control, observer, tool, memory, and task-state provider families.

## Versioning

This workspace follows the Greentic convention of keeping the root workspace version authoritative and using `version.workspace = true` in members. Shared Greentic dependencies use the `0.4` compatibility line rather than patch-pinned versions.

## Current status

The repo is still a scaffold, but the shared helper layer is now functional:

- the root binary is a placeholder entrypoint,
- the common provider crate now contains reusable provider, capability, and pack-fixture helpers,
- the engine family has shared contract helpers plus `default` and `router-lite` example directories,
- the control family has shared contract helpers plus `basic-policy` and `delegation-guard` example directories,
- the observer family has shared contract helpers plus `basic-audit` and `basic-metrics` example directories,
- the tool family has shared contract helpers plus `wasm-adapter` and `mcp-adapter` example directories,
- the short-term memory family has shared contract helpers plus `in-memory` and `redis` example directories,
- the task-store family has shared contract helpers plus `in-memory` and `redis` example directories,
- the planned category crates have not been created yet.

## CI and Releases

Local validation runs through [`ci/local_check.sh`](ci/local_check.sh). The script is intended to mirror the basic workspace checks used by Greentic Rust repos:

- formatting
- clippy
- tests
- build
- documentation

Coverage policy enforcement runs through [`greentic-dev coverage`](https://github.com/greenticai/greentic-dev) against [`coverage-policy.json`](coverage-policy.json). The nightly GitHub workflow reuses a shared Rust setup, installs `greentic-dev`, `cargo-nextest`, and `cargo-llvm-cov` with `cargo-binstall`, and fails if the policy is violated.

Lightweight performance and concurrency checks run through [`benches/perf.rs`](benches/perf.rs), [`tests/perf_scaling.rs`](tests/perf_scaling.rs), and [`tests/perf_timeout.rs`](tests/perf_timeout.rs). The Criterion harness now splits provider-extension validation from pack-manifest assembly and CBOR encoding so the hot path is easier to pinpoint. The `perf` GitHub workflow runs the workspace tests and a Criterion smoke benchmark on pull requests and pushes to the main branch.

Release automation has not been added yet; the repo currently only has bootstrap-level CI plus nightly coverage and lightweight perf enforcement.

## Examples

The `examples/pr06` directory contains the current end-to-end fixture set. It shows two documented paths:

- `oss` for in-memory providers and local component references
- `enterprise` for Redis-backed providers and OCI-style references

The fixtures cover the short-term memory, task-state, and audit observer contracts together so future bundle/setup work has a concrete reference point.
