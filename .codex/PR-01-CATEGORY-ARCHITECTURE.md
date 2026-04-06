# PR-01: Define category architecture, provider manifests, and shared helpers

## Objective
Add the shared architecture used by all provider categories.

## Scope
- shared provider manifest/types helpers
- shared capability declaration helpers
- shared self-description validation helpers
- shared config/test harness
- local development still keeps the sibling `greentic-dw` and `greentic-cap` workspaces available by path at the root level, but the implemented helper layer currently builds on the reusable `greentic-cap-types` surface and the published `0.4` Greentic crates

## Deliverables
- common provider support crate with builder and validation helpers
- example `pack.cbor` capability declaration patterns
- docs on category conventions

## Current Result
- `crates/greentic-dw-providers-common` now provides:
  - `ProviderCategory`
  - provider manifest/declaration builders
  - capability declaration builders and validation wrappers
  - pack fixture helpers that can emit sample `pack.cbor` bytes
- The helper crate is intentionally narrow: it does not implement provider runtimes, and it does not yet pull in the heavier DW manifest/workflow crates directly.
