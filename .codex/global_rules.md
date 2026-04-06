GLOBAL RULE – REPO OVERVIEW, CI, AND REUSE OF GREENTIC REPOS

For THIS REPOSITORY, you must ALWAYS:

1. Maintain `.codex/repo_overview.md` using the “Repo Overview Maintenance” routine BEFORE starting any new PR and AFTER finishing it.
2. Run `ci/local_check.sh` at the end of your work and ensure it passes, or explain precisely why it cannot be made to pass as part of this PR.
3. Prefer using existing Greentic repos/crates (interfaces, types, secrets, oauth, messaging, events, etc.) instead of reinventing types, interfaces, or behaviour locally.
4. When generating `gtpack` or `gtbundle` artifacts, always use `gtc wizard --answers` and source the schema from `gtc wizard --schema`.

Treat these as built-in prerequisites and finalisation steps for ALL work in this repo.

---

### Workflow for EVERY PR

Whenever you are asked to implement a change, feature, refactor, or bugfix, follow this workflow:

1. PRE-PR SYNC (MANDATORY)
   - Check out the target branch for this work.
   - Run the “Repo Overview Maintenance” routine:
     - Fully refresh `.codex/repo_overview.md` so it accurately reflects the current state of the repo before making any changes.
   - Show the updated `.codex/repo_overview.md` if it changed in a meaningful way.

2. IMPLEMENT THE PR
   - Apply the requested changes (code, tests, docs, configs, etc.).
   - Reuse existing Greentic crates and shared types before introducing new ones.
   - Run the appropriate build/test commands while you work and fix issues related to your changes.

3. POST-PR SYNC (MANDATORY)
   - Re-run the “Repo Overview Maintenance” routine on the updated codebase.
   - Run `ci/local_check.sh` from the repo root.
   - If it fails for reasons within scope, fix them. If it fails for reasons outside scope, capture the failing steps and explain why they remain.
   - Ensure `.codex/repo_overview.md` is consistent and up to date.

### Behavioural Rules

- Do not ask for permission to run the repo overview routine, run `ci/local_check.sh`, or reuse existing Greentic crates.
- Never leave `.codex/repo_overview.md` partially updated or inconsistent.
- Do not introduce duplicate cross-repo types or interfaces without a strong, documented justification.
- Prefer workspace-managed versions in Cargo manifests:
  - keep the root package/workspace version authoritative,
  - use `version.workspace = true` in workspace members where applicable,
  - use compatibility ranges like `0.4` for shared Greentic crates instead of patch-pinned versions such as `0.4.17`.
