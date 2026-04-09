# observer

Observer provider family.

The observer family now contains one shared contract crate plus two backend crates:

- `observer/core`
- `observer/basic-audit`
- `observer/basic-metrics`

The current shared contracts are:

- capability URIs `cap://dw.observer.audit` and `cap://dw.observer.metrics`
- pack capability ids `greentic.cap.observer.audit` and `greentic.cap.observer.metrics`
- provider types `dw.observer.basic-audit` and `dw.observer.basic-metrics`

Implementation notes:

- `greentic-dw-observer` defines the shared observer trait, event model, and report model.
- `greentic-dw-observer-basic-audit` stores a per-tenant append-only audit trail.
- `greentic-dw-observer-basic-metrics` aggregates per-tenant event counts and totals.

Example bundle-resolution data lives in [`example-bundle-resolution.json`](example-bundle-resolution.json).
