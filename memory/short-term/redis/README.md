# memory/short-term/redis

Redis-backed short-term memory provider crate.

This backend now contains real Rust code:

- crate: `greentic-dw-memory-redis`
- shared contract crate: `greentic-dw-memory`
- capability URI: `cap://dw.memory.short-term`
- pack capability id: `greentic.cap.memory.short-term`
- operations: `memory.get`, `memory.put`, `memory.delete`, `memory.clear`

The provider reuses `greentic-state`'s `RedisStateStore` and exposes the canonical provider
declaration and pack manifest for the Redis variant.
