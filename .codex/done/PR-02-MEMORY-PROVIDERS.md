# PR-02: Add initial memory providers

## Objective
Ship first memory providers, including the requested short-term memory examples.

## Providers
### short-term memory
- in-memory
- redis

## Capability
Primary capability:
- `cap://dw.memory.short-term`

## Contract direction
Expected operations:
- `memory.get`
- `memory.put`
- `memory.delete`
- `memory.clear`

## Deliverables
- `memory/short-term/in-memory`
- `memory/short-term/redis`
- sample pack declarations in `pack.cbor`-style helper output
- tests proving both satisfy the same capability contract

## Current Result
- The shared helper crate now exposes:
  - `ShortTermMemoryVariant` for `in-memory` and `redis`
  - the shared capability URI `cap://dw.memory.short-term`
  - the pack capability id `greentic.cap.memory.short-term`
  - the standard operations `memory.get`, `memory.put`, `memory.delete`, `memory.clear`
  - sample provider declarations, capability declarations, pack manifests, and CBOR encoding helpers for both variants
- The repository also includes the `memory/short-term/in-memory` and `memory/short-term/redis` documentation directories to anchor the first provider family.
