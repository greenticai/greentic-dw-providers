# Delegation Providers

Delegation providers choose which agent or execution target should handle a plan step.

The shared contract uses the `dw.delegation.*` provider namespace and the `cap://dw.delegation`
capability family.

This family currently includes:

- `core` for delegation requests, outcomes, and rationale entries
- `static-router` for config-driven deterministic routing
- `capability-match` for routing based on declared agent capabilities and schema compatibility

Concrete routing backends share one normalized request model, fanout mode, and rationale format.

## Overview

Use the delegation family when a plan step may be routed to one or more external agents, workers,
or execution lanes. Every backend operates on the shared routing contract from `delegation/core`:

- `DelegationRequest` carries the step kind, goal, optional schema, tags, capabilities, visible
  targets, and policy state
- `DelegationTarget` describes a routable target with capabilities, schemas, tags, and availability
- `DelegationDecision` returns selected targets, final fanout mode, and human-readable rationale

Choose a router by how explicit the matching logic should be:

- `static-router` for checked-in deterministic rules over step kind, goal patterns, schema, and tags
- `capability-match` for selecting from declared target capabilities and schema compatibility

## Config Reference

Common runtime inputs:

- `step_kind`: required stable routing key from the planning layer
- `goal`: optional free-form description to disambiguate similar steps
- `output_schema`: optional schema id expected from the selected target
- `tags`: routing hints such as `research`, `parallel`, or `high-trust`
- `required_capabilities`: hard requirements for target selection
- `available_targets`: visible candidate targets at decision time
- `fanout`: `none`, `single`, or `parallel`
- `control_allowed`: whether policy still permits delegation

Router-specific configuration:

- `delegation/static-router`
  - ordered `routes`
  - each route may match on `step_kind`, `output_schema`, `tags`, and goal substring patterns
  - each route names explicit `target_ids` and a final `fanout`
- `delegation/capability-match`
  - no static route table
  - selection is based on target `available`, capability coverage, schema support, and tag fit

## Example Manifest Snippet

```json
{
  "providers": [
    {
      "id": "delegate-static",
      "type": "dw.delegation.static-router",
      "capabilities": ["cap://dw.delegation.route"],
      "component": {
        "ref": "oci://ghcr.io/greenticai/packs/dw/delegation/static-router-pack:0.5.4"
      },
      "config": {
        "routes": [
          {
            "step_kind": "delegate",
            "tags": ["research"],
            "target_ids": ["agent-blue"],
            "fanout": "single"
          }
        ]
      }
    }
  ]
}
```

## Expected Runtime Behavior

- Delegation should happen after planning has emitted stable step kinds.
- If `control_allowed` is `false`, providers must not route around policy denial.
- Decisions are expected to include rationale entries so later review can explain why a target was
  selected or why nothing matched.
- Static routing is deterministic for the same inputs.
- Capability matching is also deterministic because targets are ranked and returned in stable order.

## Troubleshooting

- If routing fails with `delegation denied by control policy`, fix the upstream policy or skip
  delegation for that step.
- If a static rule resolves to an unavailable target, either expose that target in
  `available_targets` or change the route table.
- If no targets are selected, inspect `step_kind`, `output_schema`, tags, and required
  capabilities first; one mismatched field is usually enough to eliminate all candidates.
- If a `single` route names more than one target, correct the route config or switch the fanout to
  `parallel`.
