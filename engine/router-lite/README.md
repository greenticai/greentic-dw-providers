# Engine Router Lite

Router/planner-lite engine provider crate.

The current shared contract uses:

- provider type `dw.engine.router-lite`
- capability URI `cap://dw.engine.router`
- pack capability id `greentic.cap.engine.router`

This backend now contains real Rust code and performs lightweight rule-based route selection
with a simple execution plan outline.
