# Control Delegation Guard

Delegation guard control provider crate.

The shared contract uses:

- provider type `dw.control.delegation-guard`
- capability URI `cap://dw.control.delegation-guard`
- pack capability id `greentic.cap.control.delegation-guard`

This backend now contains real Rust code and blocks delegation requests unless the
request explicitly carries `allow_delegation=true`.
