# control

Control provider family.

The control family now contains one shared contract crate plus two backend crates:

- `control/core`
- `control/basic-policy`
- `control/delegation-guard`

The current shared contracts are:

- capability URIs `cap://dw.control.basic` and `cap://dw.control.delegation-guard`
- pack capability ids `greentic.cap.control.basic` and `greentic.cap.control.delegation-guard`
- provider types `dw.control.basic-policy` and `dw.control.delegation-guard`

Implementation notes:

- `greentic-dw-control` defines the shared control trait, request model, and decision model.
- `greentic-dw-control-basic-policy` performs simple allow/deny evaluation.
- `greentic-dw-control-delegation-guard` blocks delegation unless it is explicitly allowed.

Example bundle-resolution data lives in [`example-bundle-resolution.json`](example-bundle-resolution.json).
