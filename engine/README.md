# Engine Providers

This directory anchors the engine provider family planned by the workspace.

The shared helper crate currently defines the first engine contract:

- `cap://dw.engine.default`
- `cap://dw.engine.router`

The current contract split is:

- `engine/default/` for the default/simple engine
- `engine/router-lite/` for the router/planner-lite engine

The provider helper crate now exposes:

- `EngineVariant`
- `engine_capability_uri`
- `engine_capability_id`
- `engine_pack_capability_id`
- `engine_provider_decl`
- `engine_provider_extension_inline`
- `engine_capability_declaration`
- `engine_pack_manifest`
- `engine_pack_manifest_cbor`
