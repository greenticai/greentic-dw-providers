# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## What this is

`greentic-dw-providers` is the Rust workspace that hosts **provider implementations** for the Greentic Digital Worker contracts defined in `greentic-dw`. Each provider family lives in its own subtree (`llm/`, `memory/`, `state/`, `engine/`, `control/`, `planning/`, `workspace/`, `delegation/`, `reflection/`, `context/`, `observer/`, `tool/`) and consumes the core contracts via the `greentic-dw` and `greentic-cap` published crate lines (the `0.5` line at time of writing).

The shared helper crate `crates/greentic-dw-providers-common` enforces the naming conventions and provides reusable fixtures for capability declarations, manifests, runtime refs, and bundle/setup scenarios.

The root binary (`src/main.rs`) is a workspace banner — **not** a runtime entrypoint. Don't grow it into one.

## Build, test, verify

Canonical local CI (run from repo root):

```bash
bash ci/local_check.sh
```

That script runs, in order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test  --workspace --lib`
4. `cargo test  --workspace --tests`
5. `cargo test  -p greentic-dw-providers --test provider_composition --test provider_golden`
6. `cargo build --workspace --all-features`
7. `cargo doc   --workspace --no-deps --all-features`
8. `bash ci/gtpacks.sh validate` — validates wizard manifests under `packs/`

The two named integration tests are load-bearing — `provider_composition` and `provider_golden` exercise the OSS / enterprise bundle/setup fixtures end-to-end.

Day-to-day:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  -p greentic-dw-llm-anthropic <name> -- --nocapture
cargo test  --workspace --all-features
```

Toolchain pinned to **Rust 1.95.0** via canonical `rust-toolchain.toml`. Edition 2024.

CI mirrors `local_check.sh` and adds reusable coverage (`_reusable_coverage.yml`, `coverage.yml`, `nightly-coverage.yml`), `perf.yml`, and `publish.yml` (gtpack publish to GHCR under `packs/dw/<dw-type>/<dw-name>-pack`).

## Workspace layout — provider families

Each family has a `core` crate (the contract) plus zero or more backend crates (the implementations). Current state:

| Family | Core | Backends |
|--------|------|----------|
| **llm** | `llm/core` | `anthropic`, `azure-openai`, `bedrock`, `gemini`, `openai`, `openai-compatible`, `nvidia-nim` |
| **memory** (short-term) | `memory/short-term/core` | `in-memory`, `redis` |
| **state** (task-store) | `state/task-store/core` | `in-memory`, `redis` |
| **engine** | `engine/core` | `default`, `router-lite` |
| **control** | `control/core` | `basic-policy`, `delegation-guard` |
| **observer** | `observer/core` | `basic-audit`, `basic-metrics` |
| **tool** | `tool/core` | `component-adapter`, `mcp-adapter` |
| **planning** | `planning/core` | `static`, `llm-outline` |
| **reflection** | `reflection/core` | `rules`, `schema-check`, `llm-critic` |
| **context** | `context/core` | `static`, `compressor`, `retrieval` |
| **workspace** | `workspace/core` | `in-memory`, `fs` |
| **delegation** | `delegation/core` | `static-router`, `capability-match` |

Plus `crates/greentic-dw-providers-common` (shared helpers) and `packs/gtpacks.manifest.json` (release manifest for gtpack scaffolds).

### Naming conventions (enforced by `greentic-dw-providers-common`)

- Provider type IDs: `dw.<category>.<provider-name>`
- Runtime capability URIs: `cap://dw.<category>.<capability>`
- Pack capability IDs: `greentic.cap.<category>.<capability>`

Exception — the `llm` family flattens to `cap://dw.llm` and `greentic.cap.llm` (no per-provider sub-segment). Use the builder helpers in `greentic-dw-providers-common` rather than hand-rolling these strings.

## Source-of-truth order

1. `.codex/repo_overview.md` and `.codex/global_rules.md` (both present and authoritative).
2. `crates/greentic-dw-providers-common/src/` for the canonical naming/fixture helpers.
3. Each family's `core/src/` for the contract.
4. `tests/` (especially `provider_composition.rs` and `provider_golden.rs`) for end-to-end fixtures.
5. `README.md` — accurate but slower-moving than `.codex/repo_overview.md`.

## `.codex/` workflow (mandatory)

`.codex/global_rules.md` is explicit and binding for this repo:

1. **PRE-PR sync** — refresh `.codex/repo_overview.md` against current state.
2. **Implement** — reuse Greentic crates first; do not invent new core types.
3. **POST-PR sync** — refresh again, run `bash ci/local_check.sh`, document any out-of-scope failures in the PR.
4. When generating gtpack/gtbundle artifacts, source schemas via `gtc wizard --schema` and feed them through `gtc wizard --answers`.

Behavioural rules from `global_rules.md` worth highlighting:

- Do not ask for permission to run the overview routine, run `local_check.sh`, or reuse existing Greentic crates.
- Never leave `.codex/repo_overview.md` partially updated.
- Prefer workspace-managed versions (`version.workspace = true`) and compatibility ranges (e.g. `0.4`) over patch-pinned versions for shared Greentic crates.

## Reuse-first

This repo is the **largest reuse-first surface** in the Greentic stack — its whole job is to plug shared contracts into concrete backends. Before adding a new type, interface, or helper, check:

- `greentic-types`, `greentic-interfaces`, `greentic-pack`, `greentic-state` — shared cross-repo DTOs and pack patterns.
- `greentic-cap-types` / `greentic-cap-*` — capability declarations and pack capability ids.
- `greentic-dw` (`0.5` line) — the DW contracts these providers implement. **Do not redefine** DW core types here.
- `greentic-dw-providers-common` — the local shared helpers.

Forking or duplicating a shared model requires documented justification in the PR.

## Style guardrails

- English only in source, tests, comments, commits, tracing.
- `#![forbid(unsafe_code)]` at crate roots.
- No `unwrap()` / `panic!()` in production paths — use `anyhow`/`thiserror`.
- Conventional Commits.
- **Do not** add Claude co-authorship trailers or "Generated with Claude Code" lines on commits or PR bodies.
- `Cargo.lock` is committed; CI runs `--locked`.
- Husky / `.githooks/` may run `local_check.sh` — never bypass with `--no-verify`.

## Branching and release

`main` is default; `develop` exists. Releases publish gtpacks to GHCR under `packs/dw/<dw-type>/<dw-name>-pack` via `publish.yml`. The OCI tag convention is `:stable` (see meta-workspace memory on the May 2026 `:latest` → `:stable` migration). `tag-on-version-bump.yml` only fires on a version diff in `Cargo.toml` — bump the version rather than re-tagging when a release workflow needs to re-run.
