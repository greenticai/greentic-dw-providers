# Control Basic Policy

Basic policy control provider crate.

The shared contract uses:

- provider type `dw.control.basic-policy`
- capability URI `cap://dw.control.basic`
- pack capability id `greentic.cap.control.basic`

This backend now contains real Rust code and evaluates simple allow/deny policy rules
based on the requested action and policy attributes.
