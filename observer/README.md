# observer

Observer provider family anchors.

The shared helper crate currently defines two observer variants:

- `basic-audit`
- `basic-metrics`

The current shared contracts are:

- capability URIs `cap://dw.observer.audit` and `cap://dw.observer.metrics`
- pack capability ids `greentic.cap.observer.audit` and `greentic.cap.observer.metrics`
- provider types `dw.observer.basic-audit` and `dw.observer.basic-metrics`

Example bundle-resolution data lives in [`example-bundle-resolution.json`](example-bundle-resolution.json).
