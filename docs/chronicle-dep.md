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

All four crates are pinned to **tag `v0.1.0`**, commit rev
`3faf8903bd2a1d3d9b3c71a31a3c2ed771f13642`, in the root `Cargo.toml`:

```toml
chronicle-core         = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
chronicle-driver-neo4j = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
chronicle-llm-openai   = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
chronicle-testkit      = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
```

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
   (e.g., `v0.2.0`) pointing at the target commit.
2. In the root `Cargo.toml` of this repo: update all four `chronicle-*` entries to
   the new tag.
3. Run `cargo update -p chronicle-core -p chronicle-driver-neo4j -p chronicle-llm-openai -p chronicle-testkit`
   to refresh `Cargo.lock` to the new revision.
4. Run `bash ci/local_check.sh` to verify nothing is broken.
5. Open a PR targeting `research` with the version bump.
