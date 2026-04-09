# memory/short-term/in-memory

In-memory short-term memory provider crate.

This backend now contains real Rust code:

- crate: `greentic-dw-memory-in-memory`
- shared contract crate: `greentic-dw-memory`
- capability URI: `cap://dw.memory.short-term`
- pack capability id: `greentic.cap.memory.short-term`
- operations: `memory.get`, `memory.put`, `memory.delete`, `memory.clear`

The provider delegates storage to `greentic-state`'s `InMemoryStateStore` and exposes the
canonical provider declaration and pack manifest for the in-memory variant.
