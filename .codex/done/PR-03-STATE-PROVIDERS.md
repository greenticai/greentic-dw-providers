# PR-03: Add initial task-state providers

## Objective
Ship first task-state / resume providers.

## Providers
- in-memory task-store
- redis task-store

## Capability
- `cap://dw.state.task-store`

## Responsibilities
- checkpointing
- lifecycle state persistence
- resume lookup
- pending await metadata

## Deliverables
- `state/task-store/in-memory`
- `state/task-store/redis`
- integration tests with greentic-dw runtime fixtures

## Current Result
- The shared helper crate now exposes:
  - `TaskStoreVariant` for `in-memory` and `redis`
  - the shared capability URI `cap://dw.state.task-store`
  - the pack capability id `greentic.cap.state.task-store`
  - the standard operations `state.load`, `state.save`, `state.list`
  - state-key and state-path helpers for checkpoint and resume metadata
  - sample provider declarations, capability declarations, pack manifests, and CBOR encoding helpers for both variants
- The repository also includes the `state/task-store/in-memory` and `state/task-store/redis` documentation directories to anchor the first state provider family.
