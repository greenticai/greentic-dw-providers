# memory/short-term/redis

Placeholder for the Redis-backed short-term memory provider.

This provider is expected to implement the same shared short-term memory contract as the in-memory variant:

- capability URI: `cap://dw.memory.short-term`
- pack capability id: `greentic.cap.memory.short-term`
- operations: `memory.get`, `memory.put`, `memory.delete`, `memory.clear`

The helper crate includes sample manifest and pack fixtures for this variant.
