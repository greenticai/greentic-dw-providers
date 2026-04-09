# memory

Memory provider family for Greentic DW providers.

The first implemented provider set is short-term memory, with two backend crates:

- `memory/short-term/core`
- `memory/short-term/in-memory`
- `memory/short-term/redis`

Both backends use the same shared short-term capability contract:

- capability URI: `cap://dw.memory.short-term`
- pack capability id: `greentic.cap.memory.short-term`
- operations: `memory.get`, `memory.put`, `memory.delete`, `memory.clear`

Implementation notes:

- `greentic-dw-memory` provides the shared short-term memory trait, config model, and a generic `StateStore`-backed implementation.
- `greentic-dw-memory-in-memory` binds that contract to `greentic-state`'s `InMemoryStateStore`.
- `greentic-dw-memory-redis` binds that contract to `greentic-state`'s `RedisStateStore`.
- `greentic-dw-providers-common` still exposes the canonical provider declaration and pack manifest helpers for both variants.
