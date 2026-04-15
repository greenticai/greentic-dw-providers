# PR-05: Add initial control, observer, and tool adapter providers

## Objective
Add enough reference providers to make end-to-end examples realistic.

## Providers
### control
- basic-policy
- delegation-guard

### observer
- basic-audit
- basic-metrics

### tool
- component-adapter
- mcp-adapter

## Example capabilities
- `cap://dw.control.basic`
- `cap://dw.control.delegation-guard`
- `cap://dw.observer.audit`
- `cap://dw.observer.metrics`
- `cap://dw.tool.component`
- `cap://dw.tool.mcp`

## Deliverables
- category implementations
- example configs
- example bundle resolution data

## Current Result
- The shared helper crate now exposes:
  - `ControlVariant` for `basic-policy` and `delegation-guard`
  - `ObserverVariant` for `basic-audit` and `basic-metrics`
  - `ToolVariant` for `component-adapter` and `mcp-adapter`
  - the shared capability URIs, pack capability ids, provider declarations, capability declarations, pack manifests, and CBOR encoding helpers for all three families
  - pack-capability selection helpers for each family
- The repository also includes the `control/`, `observer/`, and `tool/` documentation trees plus example bundle-resolution JSON files for each family.
