# PR-12: Workspace Providers

## Title
feat(providers/workspace): add in-memory and filesystem workspace providers

## Dependencies
- greentic-dw PR-02

## Scope
Implement:
- `workspace/in-memory`
- `workspace/fs`

## Concrete work

### in-memory
- `DashMap` or equivalent store
- immutable version chain
- scoped listing
- derived-from linkage

### fs
- root directory config
- one folder per workspace scope
- one metadata file and one content file per artifact version
- safe path normalization
- content-addressable checksum stored in metadata

## Tests
- version history preserved
- fs provider rejects path traversal
- list artifacts by scope works
- artifact provenance survives reload

## Acceptance criteria
- Workspace is usable without any external service.
- File-backed provider is deterministic and inspectable.
