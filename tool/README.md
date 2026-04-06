# tool

Tool provider family anchors.

The shared helper crate currently defines two tool variants:

- `wasm-adapter`
- `mcp-adapter`

The current shared contracts are:

- capability URIs `cap://dw.tool.wasm` and `cap://dw.tool.mcp`
- pack capability ids `greentic.cap.tool.wasm` and `greentic.cap.tool.mcp`
- provider types `dw.tool.wasm-adapter` and `dw.tool.mcp-adapter`

Example bundle-resolution data lives in [`example-bundle-resolution.json`](example-bundle-resolution.json).
