# memory

Memory provider family for Greentic DW providers.

The first planned provider set is short-term memory, with two backend variants:

- `memory/short-term/in-memory`
- `memory/short-term/redis`

Both variants use the same shared short-term capability contract:

- capability URI: `cap://dw.memory.short-term`
- pack capability id: `greentic.cap.memory.short-term`
- operations: `memory.get`, `memory.put`, `memory.delete`, `memory.clear`

The shared helper crate exposes fixtures for both variants so future provider crates can reuse the same contract and pack metadata.
