# PR-04: Add initial engine providers

## Objective
Ship one or two basic engine providers.

## Providers
- default/simple engine
- router/planner-lite engine

## Capabilities
- `cap://dw.engine.default`
- `cap://dw.engine.router`

## Deliverables
- `engine/default`
- `engine/router-lite`
- examples showing selection through capability resolution

## Current Result
- The shared helper crate now exposes:
  - `EngineVariant` for `default` and `router-lite`
  - the shared capability URIs `cap://dw.engine.default` and `cap://dw.engine.router`
  - the pack capability ids `greentic.cap.engine.default` and `greentic.cap.engine.router`
  - the shared operations `engine.decide` and `engine.route`
  - provider declarations, capability declarations, pack manifests, CBOR encoding helpers, and a pack-capability selection helper for both variants
- The repository also includes the `engine/default` and `engine/router-lite` documentation directories to anchor the first engine provider family.
