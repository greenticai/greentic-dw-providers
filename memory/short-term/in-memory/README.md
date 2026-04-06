# memory/short-term/in-memory

Placeholder for the in-memory short-term memory provider.

This provider is expected to implement the shared short-term memory contract:

- capability URI: `cap://dw.memory.short-term`
- pack capability id: `greentic.cap.memory.short-term`
- operations: `memory.get`, `memory.put`, `memory.delete`, `memory.clear`

The helper crate includes sample manifest and pack fixtures for this variant.
