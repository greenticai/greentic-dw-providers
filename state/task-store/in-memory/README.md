# state/task-store/in-memory

Placeholder for the in-memory task-store provider.

This provider is expected to implement the shared task-store contract:

- capability URI: `cap://dw.state.task-store`
- pack capability id: `greentic.cap.state.task-store`
- operations: `state.load`, `state.save`, `state.list`

The helper crate includes sample manifest and pack fixtures for this variant.
