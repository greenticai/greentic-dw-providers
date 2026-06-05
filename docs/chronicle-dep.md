# Private Chronicle Dependency

## Why this dependency exists

The `memory/long-term/chronicle` crate family wraps the Greentic-private repository
`greentic-biz/greentic-chronicle-ext`, which provides the knowledge-graph memory
primitives (episode ingestion, entity extraction, bi-temporal storage, and retrieval)
used by the long-term memory provider family in this workspace.

Three crates are pulled from that repo:

| Workspace alias          | Source crate           | Purpose                                      |
|--------------------------|------------------------|----------------------------------------------|
| `chronicle-core`         | `chronicle-core`       | Core traits, episode/entity models, errors   |
| `chronicle-driver-neo4j` | `chronicle-driver-neo4j` | Neo4j graph-store driver                   |
| `chronicle-llm-openai`   | `chronicle-llm-openai` | OpenAI-backed entity extractor               |
| `chronicle-testkit`      | `chronicle-testkit`    | Test fixtures (dev/test builds only)         |

All four are pinned to **tag `v0.1.0`**, commit rev
`3faf8903bd2a1d3d9b3c71a31a3c2ed771f13642`, in the root `Cargo.toml`:

```toml
chronicle-core         = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
chronicle-driver-neo4j = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
chronicle-llm-openai   = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
chronicle-testkit      = { git = "https://github.com/greentic-biz/greentic-chronicle-ext.git", tag = "v0.1.0" }
```

## Required secret: CHRONICLE_REPO_TOKEN

CI runners have no implicit access to `greentic-biz/greentic-chronicle-ext`.
Every `cargo` command that resolves the workspace graph (build, test, clippy, doc,
publish) will fail at `git fetch` unless a valid token is available.

The token must be a **fine-grained PAT** (or GitHub App installation token) with
the minimum scope:

| Permission   | Level     | Reason                               |
|--------------|-----------|--------------------------------------|
| `contents`   | read-only | Clone / fetch the private repository |

No other permissions are needed. The token should be scoped to
`greentic-biz/greentic-chronicle-ext` only.

## Workflows that reference this secret

The following workflows in this repository run `cargo` commands directly and have
been wired with the `Configure git auth for private chronicle dep` step plus
`CARGO_NET_GIT_FETCH_WITH_CLI: "true"`:

| Workflow file                     | Jobs wired                                  |
|-----------------------------------|---------------------------------------------|
| `.github/workflows/perf.yml`      | `perf`                                      |
| `.github/workflows/publish.yml`   | `ci`, `publish-crates`, `build-gtpacks`     |
| `.github/workflows/_reusable_coverage.yml` | `coverage` (consumed by `coverage.yml` and `nightly-coverage.yml`) |

### External / reusable workflows (cannot be edited in this repo)

The following workflows delegate entirely to `greenticai/.github` via `uses:` and
therefore cannot be patched here. They will also fail on `cargo` commands that
trigger a dependency fetch unless `CHRONICLE_REPO_TOKEN` is made available to the
called workflow (either by `secrets: inherit` — which is already present on several
callers — or by the `greenticai/.github` workflow being updated to pass the secret
through its own git-auth step):

| Workflow file                            | Called workflow                                                   | Has `secrets: inherit`? |
|------------------------------------------|-------------------------------------------------------------------|-------------------------|
| `.github/workflows/ci.yml`               | `greenticai/.github/.github/workflows/host-crate-ci.yml@main`    | No (uses: only)         |
| `.github/workflows/codeql.yml`           | `greenticai/.github/.github/workflows/codeql.yml@main`           | No (uses: only)         |
| `.github/workflows/branch-invariants.yml`| `greenticai/.github/.github/workflows/assert-branch-invariants.yml@main` | No               |
| `.github/workflows/tag-on-version-bump.yml` | `greenticai/.github/.github/workflows/tag-on-version-bump.yml@main` | Yes                  |
| `.github/workflows/codex-security-fix.yml` | `greenticai/.github/.github/workflows/codex-security-fix.yml@main` | Yes                  |
| `.github/workflows/codex-semver-fix.yml` | `greenticai/.github/.github/workflows/codex-semver-fix.yml@main` | Yes                   |
| `.github/workflows/dependabot-automerge.yml` | `greenticai/.github/.github/workflows/dependabot-automerge.yml@main` | Yes               |
| `.github/workflows/dependency-review.yml` | `greenticai/.github/.github/workflows/dependency-review.yml@main` | Yes                  |

For callers that pass `secrets: inherit`, the `CHRONICLE_REPO_TOKEN` org/repo secret
will be forwarded automatically once devops creates it. For callers that do not
(notably `ci.yml` and `codeql.yml`), the `greenticai/.github` reusable workflows
themselves need a `Configure git auth for private chronicle dep` step added — that
change must be made in the `greenticai/.github` repository by devops.

## Local development requirement

Developers need read access to `greentic-biz/greentic-chronicle-ext` to build this
workspace locally. Either of the following setups works:

- **HTTPS via gh CLI**: `gh auth login` — Cargo will delegate to the `gh` credential
  helper automatically when `CARGO_NET_GIT_FETCH_WITH_CLI=true` is set (or when git
  is configured to use the gh credential helper globally).
- **SSH**: Add your SSH key to GitHub and configure git to rewrite HTTPS URLs to SSH:
  ```
  git config --global url."git@github.com:greentic-biz/".insteadOf "https://github.com/greentic-biz/"
  ```

If `cargo build` fails with a `git fetch` authentication error, verify your GitHub
account has been granted access to the private repo.

## Bump procedure

When a new version of `greentic-chronicle-ext` needs to be adopted:

1. In the `greentic-biz/greentic-chronicle-ext` repo: create and push the new tag
   (e.g., `v0.2.0`) with the target commit.
2. In the root `Cargo.toml` of this repo: update all four `chronicle-*` entries to
   the new tag.
3. Run `cargo update -p chronicle-core -p chronicle-driver-neo4j -p chronicle-llm-openai -p chronicle-testkit`
   to refresh `Cargo.lock` to the new revision.
4. Run `bash ci/local_check.sh` to verify nothing is broken.
5. Open a PR targeting `research` with the version bump.

---

## HANDOFF — Action required by devops

**Create the `CHRONICLE_REPO_TOKEN` Actions secret before merging this PR. CI will
fail on `git fetch` for every cargo-running job until the secret exists.**

Steps:

1. Generate a fine-grained PAT (or GitHub App installation token) with
   `contents: read` on `greentic-biz/greentic-chronicle-ext` only.
2. Add it as a repository (or organization) Actions secret named exactly
   `CHRONICLE_REPO_TOKEN` on this repository (`greentic-dw-providers`).
3. For the `greenticai/.github`-hosted reusable workflows (`ci.yml` → `host-crate-ci.yml`,
   `codeql.yml` → `codeql.yml`) that do NOT pass `secrets: inherit`, a corresponding
   git-auth step must be added in the `greenticai/.github` repo so those callers can
   also resolve the dependency. Alternatively, add `secrets: inherit` to those two
   caller workflows in this repo — but that requires the called workflow to accept
   the secret explicitly.
