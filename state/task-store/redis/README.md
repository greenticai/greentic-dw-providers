# state/task-store/redis

Redis-backed task-store provider crate.

This backend now contains real Rust code:

- crate: `greentic-dw-task-store-redis`
- shared contract crate: `greentic-dw-task-store`
- capability URI: `cap://dw.state.task-store`
- pack capability id: `greentic.cap.state.task-store`
- operations: `state.load`, `state.save`, `state.list`

The provider reuses `greentic-state`'s `RedisStateStore` and exposes the canonical provider
declaration and pack manifest for the Redis variant.
