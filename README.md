# greentic-dw-providers

Greentic digital worker provider workspace scaffold.

This repository is the starting point for provider implementations that will plug into the Greentic digital worker ecosystem. The intended provider families are:

- llm
- memory
- state
- engine
- control
- planning
- workspace
- delegation
- reflection
- context
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
- llm family fixtures and feature-profile helpers
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

The new `llm` family is the current exception:

- runtime capability URI uses `cap://dw.llm`
- shared pack capability id uses `greentic.cap.llm`

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

- `llm/`
  - `core/`
  - `anthropic/`
  - `azure-openai/`
  - `bedrock/`
  - `gemini/`
  - `openai/`
  - `openai-compatible/`
  - `nvidia-nim/`
- `engine/`
  - `default/`
  - `router-lite/`
- `planning/`
  - `core/`
  - `static/`
  - `llm-outline/`
- `workspace/`
  - `core/`
  - `in-memory/`
  - `fs/`
- `delegation/`
  - `core/`
  - `static-router/`
  - `capability-match/`
- `reflection/`
  - `core/`
  - `schema-check/`
  - `rules/`
  - `llm-critic/`
- `context/`
  - `core/`
  - `static/`
  - `retrieval/`
  - `compressor/`
- `control/`
  - `basic-policy/`
  - `delegation-guard/`
- `observer/`
  - `basic-audit/`
  - `basic-metrics/`
- `tool/`
  - `component-adapter/`
  - `mcp-adapter/`
- `memory/`
  - `short-term/in-memory/`
  - `short-term/redis/`
- `state/`
  - `task-store/in-memory/`
  - `task-store/redis/`

The category roots are still placeholders, but `llm/core/...`, `llm/anthropic/...`, `llm/azure-openai/...`, `llm/bedrock/...`, `llm/gemini/...`, `llm/openai/...`, `llm/openai-compatible/...`, `engine/default/...`, `engine/router-lite/...`, `planning/core/...`, `planning/static/...`, `planning/llm-outline/...`, `workspace/core/...`, `workspace/in-memory/...`, `workspace/fs/...`, `delegation/core/...`, `delegation/static-router/...`, `delegation/capability-match/...`, `reflection/core/...`, `reflection/schema-check/...`, `reflection/rules/...`, `reflection/llm-critic/...`, `context/core/...`, `context/static/...`, `context/retrieval/...`, `context/compressor/...`, `control/basic-policy/...`, `control/delegation-guard/...`, `observer/basic-audit/...`, `observer/basic-metrics/...`, `tool/component-adapter/...`, `tool/mcp-adapter/...`, `memory/short-term/...`, and `state/task-store/...` now carry concrete contract crates or documentation for the first LLM, engine, planning, workspace, delegation, reflection, context, control, observer, tool, memory, and task-state provider families.

## Deep-Agent Family Guides

The planning, workspace, delegation, reflection, and context families now also include dedicated
authoring guides so a developer can wire them without reading source first:

- [planning/README.md](planning/README.md)
- [workspace/README.md](workspace/README.md)
- [delegation/README.md](delegation/README.md)
- [reflection/README.md](reflection/README.md)
- [context/README.md](context/README.md)

## Versioning

This workspace follows the Greentic convention of keeping the root workspace version authoritative and using `version.workspace = true` in members. Shared Greentic dependencies use the `0.4` compatibility line rather than patch-pinned versions.

## Current status

The repo is still a scaffold, but the shared helper layer is now functional:

- the root binary is a placeholder entrypoint,
- the common provider crate now contains reusable provider, capability, and pack-fixture helpers,
- the llm family now has a normalized shared contract crate plus canonical provider, capability, and feature-profile helpers,
- the llm family helper surface now also includes shared wizard QA descriptor types for provider-owned setup questions,
- the shared helper crate now also carries cross-provider LLM conformance tests for naming, manifests, feature flags, wizard metadata, and config schemas,
- the llm family now also has an `anthropic` backend crate for Messages API mapping, tool use, and structured outputs,
- the llm family now also has an `azure-openai` backend crate for Azure-specific deployment, auth, and Responses API execution,
- the llm family now also has a `bedrock` backend crate for AWS auth-aware Converse and ConverseStream transport over the official AWS SDK,
- the llm family now also has a `gemini` backend crate for `generateContent`, function calling, and JSON structured outputs,
- the llm family now also has a native `openai` backend crate for the OpenAI Responses API,
- the llm family now also has an `openai-compatible` backend crate for self-hosted and gateway targets,
- the llm family now also has a `nvidia-nim` backend crate for NIM-aware discovery, health-aware deployments, and mode-gated startup probing,
- the engine family has shared contract helpers plus `default` and `router-lite` example directories,
- the planning family now has shared contract helpers plus `static` and `llm-outline` planner backends,
- the workspace family now has shared artifact models plus `in-memory` and `fs` backends for deterministic local storage,
- the delegation family now has shared routing models plus `static-router` and `capability-match` backends with explicit rationale output,
- the reflection family now has deterministic schema and rules reviewers plus a strict typed `llm-critic` backend,
- the context family now has static, retrieval, and compression backends that produce deterministic context packages with explicit provenance,
- the planning, workspace, delegation, reflection, and context families now have shared core scaffold crates so future provider implementations can follow the same workspace conventions as the other families,
- the control family has shared contract helpers plus `basic-policy` and `delegation-guard` example directories,
- the observer family has shared contract helpers plus `basic-audit` and `basic-metrics` example directories,
- the tool family has shared contract helpers plus `component-adapter` and `mcp-adapter` example directories,
- the short-term memory family has shared contract helpers plus `in-memory` and `redis` example directories,
- the task-store family has shared contract helpers plus `in-memory` and `redis` example directories,
- the provider families now include real backend crates for engine, control, observer, tool, memory, and task-state work.

## CI and Releases

Local validation runs through [`ci/local_check.sh`](ci/local_check.sh). The script is intended to mirror the basic workspace checks used by Greentic Rust repos:

- formatting
- clippy
- tests
- build
- documentation

Coverage policy enforcement runs through [`greentic-dev coverage`](https://github.com/greenticai/greentic-dev) against [`coverage-policy.json`](coverage-policy.json). The nightly GitHub workflow reuses a shared Rust setup, installs `greentic-dev`, `cargo-nextest`, and `cargo-llvm-cov` with `cargo-binstall`, and fails if the policy is violated.

Lightweight performance and concurrency checks run through [`benches/perf.rs`](benches/perf.rs), [`tests/perf_scaling.rs`](tests/perf_scaling.rs), and [`tests/perf_timeout.rs`](tests/perf_timeout.rs). The Criterion harness now splits provider-extension validation from pack-manifest assembly and CBOR encoding so the hot path is easier to pinpoint. The `perf` GitHub workflow runs the workspace tests and a Criterion smoke benchmark on pull requests and pushes to the main branch.

Release automation now lives in [`.github/workflows/publish.yml`](.github/workflows/publish.yml). It runs the local CI checks on pull requests and pushes to `main`/`master`, validates release tags against the root Cargo version, publishes the shared crate to crates.io on release-capable pushes, and reserves the GHCR pack namespace `oci://ghcr.io/greenticai/packs/dw/<dw-type>/<dw-name>-pack:<version>` for future `gtpack` artifacts.

The release job now generates those `.gtpack` artifacts from `packs/gtpacks.manifest.json` using `cargo binstall gtc`, `gtc install`, `gtc wizard --schema`, and `gtc wizard --answers`, then publishes the resulting archives to GHCR with `oras` under the same `packs/dw/<dw-type>/<dw-name>-pack` namespace. The manifest now covers the implemented deep-agent families too, including `planning`, `workspace`, `delegation`, `reflection`, and `context`.

## Examples

The `examples/pr06` directory contains the current end-to-end fixture set. It shows two documented paths:

- `oss` for in-memory providers and local component references
- `enterprise` for Redis-backed providers and OCI-style references

The fixtures cover the short-term memory, task-state, and audit observer contracts together so future bundle/setup work has a concrete reference point.
