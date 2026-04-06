# PR-06 Examples

This directory contains the end-to-end provider fixtures for the repository.

The examples model a Digital Worker that needs:

- short-term memory
- task-state persistence
- audit observation

Two strategies are documented:

- `oss` using in-memory providers and local component refs
- `enterprise` using Redis-backed providers and OCI-style refs

The current fixtures are:

- `component-dw.oss.bundle.json`
- `component-dw.oss.setup.json`
- `component-dw.oss.resolution.json`
- `component-dw.enterprise.bundle.json`
- `component-dw.enterprise.setup.json`
- `component-dw.enterprise.resolution.json`

These files are documentation-only examples and are meant to mirror the bundle/setup flow used by the Greentic DW workspace and the shared capability model from `greentic-cap`.
