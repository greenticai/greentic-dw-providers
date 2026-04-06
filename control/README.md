# control

Control provider family anchors.

The shared helper crate currently defines two control variants:

- `basic-policy`
- `delegation-guard`

The current shared contracts are:

- capability URIs `cap://dw.control.basic` and `cap://dw.control.delegation-guard`
- pack capability ids `greentic.cap.control.basic` and `greentic.cap.control.delegation-guard`
- provider types `dw.control.basic-policy` and `dw.control.delegation-guard`

Example bundle-resolution data lives in [`example-bundle-resolution.json`](example-bundle-resolution.json).
