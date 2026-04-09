# state/task-store/in-memory

In-memory task-store provider crate.

This backend now contains real Rust code:

- crate: `greentic-dw-task-store-in-memory`
- shared contract crate: `greentic-dw-task-store`
- capability URI: `cap://dw.state.task-store`
- pack capability id: `greentic.cap.state.task-store`
- operations: `state.load`, `state.save`, `state.list`

The provider delegates storage to `greentic-state`'s `InMemoryStateStore` and exposes the
canonical provider declaration and pack manifest for the in-memory variant.
