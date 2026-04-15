# Workspace Providers

Workspace providers persist and retrieve intermediate artifacts produced by deep-agent workflows.

The shared contract uses the `dw.workspace.*` provider namespace and the `cap://dw.workspace`
capability family.

This family currently includes:

- `core` for artifact, version, and provenance models
- `in-memory` for deterministic local test and dev storage
- `fs` for inspectable file-backed storage with persisted provenance and checksums

The concrete backends share one normalized scope, artifact, and version-history contract.

## Overview

Use the workspace family when a workflow needs durable, queryable intermediate state between
planning, execution, context assembly, or reflection. Every backend implements the shared contract
from `workspace/core`:

- `WorkspaceScope`: validated namespace for a class of artifacts such as `analysis` or `review`
- `WorkspaceArtifactId`: validated identifier within a scope
- `WorkspaceArtifactVersion`: immutable version payload with provenance and checksum metadata
- `WorkspaceProvider`: list artifacts, load full history, and append a new version

Choose a backend by storage needs:

- `in-memory` for deterministic tests, fixtures, and short-lived local runs
- `fs` for inspectable local persistence with one directory per scope and artifact

## Config Reference

Shared contract rules:

- scope ids and artifact ids must not be empty
- scope ids and artifact ids reject path separators and traversal-like values such as `..`
- version ids must be non-empty and unique within one artifact history
- `derived_from` must point at an existing prior version when present

Backend-specific configuration:

- `workspace/in-memory`
  - no external config; state lives in process memory
- `workspace/fs`
  - `root_dir`: required base directory for persisted workspace state
  - each scope becomes a subdirectory under the root
  - each artifact version persists both metadata and content JSON on disk

## Example Manifest Snippet

```json
{
  "providers": [
    {
      "id": "workspace-local",
      "type": "dw.workspace.fs",
      "capabilities": ["cap://dw.workspace.store"],
      "component": {
        "ref": "oci://ghcr.io/greenticai/packs/dw/workspace/fs-pack:0.5.4"
      },
      "config": {
        "root_dir": ".greentic/workspace"
      }
    }
  ]
}
```

## Expected Runtime Behavior

- Writes are append-only at the version level; existing history is not mutated.
- Every persisted version gets a normalized content checksum.
- Provenance travels with the artifact version so later retrieval and review can explain where data
  came from.
- `in-memory` resets on process restart.
- `fs` preserves data across reloads and keeps layouts human-inspectable for debugging.

## Troubleshooting

- If a write fails with `workspace version ... already exists`, the caller attempted to reuse a
  version id inside the same artifact history.
- If a write fails with `derived_from ... does not exist in history`, create the parent version
  first or correct the provenance chain.
- If `fs` rejects an artifact or scope id, remove path separators, leading dots, or traversal-like
  segments from the identifier.
- If retrieved checksums do not match expectations, compare the serialized JSON content rather than
  source object construction order, because checksum generation uses normalized JSON bytes.
