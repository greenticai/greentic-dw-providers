# state/task-store/redis

Placeholder for the Redis-backed task-store provider.

This provider is expected to implement the same shared task-store contract as the in-memory variant:

- capability URI: `cap://dw.state.task-store`
- pack capability id: `greentic.cap.state.task-store`
- operations: `state.load`, `state.save`, `state.list`

The helper crate includes sample manifest and pack fixtures for this variant.
