# state

State provider family for Greentic DW providers.

The first implemented provider set is task-store, with one shared contract crate and two backend crates:

- `state/task-store/core`
- `state/task-store/in-memory`
- `state/task-store/redis`

All variants use the same shared task-store capability contract:

- capability URI: `cap://dw.state.task-store`
- pack capability id: `greentic.cap.state.task-store`
- operations: `state.load`, `state.save`, `state.list`

Implementation notes:

- `greentic-dw-task-store` provides the shared task-store trait, config model, record model, and a generic `StateStore`-backed implementation.
- `greentic-dw-task-store-in-memory` binds that contract to `greentic-state`'s `InMemoryStateStore`.
- `greentic-dw-task-store-redis` binds that contract to `greentic-state`'s `RedisStateStore`.
- `greentic-dw-providers-common` still exposes the canonical provider declaration, state-key helpers, backend-kind helpers, and pack manifest helpers for both variants.
