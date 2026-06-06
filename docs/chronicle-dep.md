# Chronicle Dependency

## Why this dependency exists

The `memory/long-term/chronicle` crate family wraps
`greentic-biz/greentic-chronicle-ext`, which provides the knowledge-graph memory
primitives (episode ingestion, entity extraction, bi-temporal storage, and retrieval)
used by the long-term memory provider family in this workspace.

Three crates are pulled from that repo:

| Workspace alias          | Source crate             | Purpose                                      |
|--------------------------|--------------------------|----------------------------------------------|
| `chronicle-core`         | `chronicle-core`         | Core traits, episode/entity models, errors   |
| `chronicle-driver-neo4j` | `chronicle-driver-neo4j` | Neo4j graph-store driver                     |
| `chronicle-llm-openai`   | `chronicle-llm-openai`   | OpenAI-backed entity extractor               |
| `chronicle-testkit`      | `chronicle-testkit`      | Test fixtures (dev/test builds only)         |

## Pin

All four crates are pinned to **tag `1.2.0-research`**, commit rev
`a7418dd8`, in the root `Cargo.toml`:

```toml
chronicle-core         = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "1.2.0-research" }
chronicle-driver-neo4j = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "1.2.0-research" }
chronicle-llm-openai   = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "1.2.0-research" }
chronicle-testkit      = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "1.2.0-research" }
```

The `1.2.0-research` tag carries **full graphiti-core parity** (Phase 4):
communities, saga threading, and bulk ingestion (`add_episode_bulk`,
`build_communities`, `summarize_saga`, `add_triplet`, `remove_episode`). It
points at the same content as the `v0.3.0` git tag.

### Breaking change handled in this bump

Phase 4 added three additive fields to `AddEpisodeRequest`
(`update_communities: bool`, `saga: Option<String>`,
`saga_previous_episode_uuid: Option<String>`) and an accompanying `Default`
impl. The long-term memory provider does **not** yet expose communities or saga,
so the named-field literal in `memory/long-term/chronicle/src/lib.rs` now appends
`..Default::default()`. This preserves prior behaviour exactly
(`update_communities = false`, no saga threading). The recall/ingest API surface
(`Chronicle::search`, `Chronicle::add_episode`) is otherwise unchanged.

## Public repository — no auth required

`greentic-biz/greentic-chronicle-ext` became a **public repository on 2026-06-05**.
No token or credential configuration is needed for CI or local development — Cargo's
built-in HTTPS fetch resolves the dependency without any `git config insteadOf`
wiring or extra secrets.

Previously (before 2026-06-05) the repo was private and required a
`CHRONICLE_REPO_TOKEN` fine-grained PAT. That requirement has been removed.

## Local development

`cargo build` and `cargo test` resolve the dependency over public HTTPS automatically.
No special credential setup is required beyond a standard internet connection.

## Bump procedure

When a new version of `greentic-chronicle-ext` needs to be adopted:

1. In `greentic-biz/greentic-chronicle-ext`: create and push a new tag
   (e.g., `1.2.0-research`) pointing at the target commit.
2. In the root `Cargo.toml` of this repo: update all four `chronicle-*` entries to
   the new tag.
3. Run `cargo update -p chronicle-core -p chronicle-driver-neo4j -p chronicle-llm-openai -p chronicle-testkit`
   to refresh `Cargo.lock` to the new revision.
4. Run `bash ci/local_check.sh` to verify nothing is broken.
5. Open a PR targeting `research` with the version bump.
