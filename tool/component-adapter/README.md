# Tool Component Adapter

Greentic component-backed tool adapter.

The shared contract uses:

- provider type `dw.tool.component-adapter`
- capability URI `cap://dw.tool.component`
- pack capability id `greentic.cap.tool.component`

This adapter invokes a local Greentic component directly, so one configured tool can proxy any compatible component operation.
