# state/task-store

Shared task-store contract for Greentic DW providers.

This family now contains one shared contract crate plus two backend variants:

- core
- in-memory
- redis

The shared crate owns the task-store trait, config surface, record model, and store-backed
implementation logic. Both backend crates implement the same task-state capability contract and
share the same pack metadata conventions.
