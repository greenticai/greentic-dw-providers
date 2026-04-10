# Engine Default

Default/simple engine provider crate.

The current shared contract uses:

- provider type `dw.engine.default`
- capability URI `cap://dw.engine.default`
- pack capability id `greentic.cap.engine.default`

This backend now contains real Rust code and picks the first available candidate, or a
goal-derived direct action when no candidates are supplied.
