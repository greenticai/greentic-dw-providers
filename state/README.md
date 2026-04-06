# state

State provider family for Greentic DW providers.

The first planned provider set is task-store, with two backend variants:

- `state/task-store/in-memory`
- `state/task-store/redis`

Both variants use the same shared task-store capability contract:

- capability URI: `cap://dw.state.task-store`
- pack capability id: `greentic.cap.state.task-store`
- operations: `state.load`, `state.save`, `state.list`

The shared helper crate exposes fixtures for both variants so future provider crates can reuse the same contract, state keys, and pack metadata.
