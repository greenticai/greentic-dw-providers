# Reflection Providers

Reflection providers review intermediate or final outputs and return typed outcomes.

The shared contract uses the `dw.reflection.*` provider namespace and the `cap://dw.reflection`
capability family.

This family currently includes:

- `core` for review requests, findings, and typed outcomes
- `schema-check` for deterministic schema validation
- `rules` for deterministic rule-based review
- `llm-critic` for strict typed LLM review output

Concrete reviewers share one normalized request model and typed `ReviewOutcome` contract.

## Overview

Use the reflection family when a workflow needs a structured gate before accepting, revising, or
rejecting output. Every backend implements the shared contract from `reflection/core`:

- `ReviewRequest` carries a stable request id, the subject under review, an optional schema, and
  metadata
- `ReviewFinding` carries a machine-readable code and human-readable message
- `ReviewOutcome` returns `Accept`, `Revise`, or `Reject` plus zero or more findings

Choose a reviewer by how deterministic the review must be:

- `schema-check` for shape validation against a JSON-schema-like subset
- `rules` for deterministic policy checks over content, counts, and scores
- `llm-critic` when a typed but model-driven critique is acceptable

## Config Reference

Shared runtime inputs:

- `request_id`: stable review correlation id
- `subject`: JSON value under review
- `schema`: optional validation target for deterministic reviewers
- `metadata`: optional runtime context for rules or prompt construction

Reviewer-specific configuration:

- `reflection/schema-check`
  - requires `ReviewRequest.schema`
  - supports `type`, `required`, `properties`, and `items`
- `reflection/rules`
  - configured rule list with checks such as `exists`, `contains`, `count_gte`, `score_gte`, and
    `schema_valid`
- `reflection/llm-critic`
  - injected LLM adapter
  - fixed review prompt / instructions
  - optional single retry when the first typed response is invalid

## Example Manifest Snippet

```json
{
  "providers": [
    {
      "id": "review-schema",
      "type": "dw.reflection.schema-check",
      "capabilities": ["cap://dw.reflection.review"],
      "component": {
        "ref": "oci://ghcr.io/greenticai/packs/dw/reflection/schema-check-pack:0.5.4"
      },
      "config": {}
    }
  ]
}
```

## Expected Runtime Behavior

- Reflection usually runs after planning, retrieval, or generation and before final acceptance.
- Accept outcomes should be empty or low-noise; revise/reject outcomes should explain why with
  explicit findings.
- Deterministic reviewers return the same result for the same subject and config.
- `llm-critic` may vary internally, but only typed `ReviewOutcome` values leave the provider.
- Missing required reviewer inputs, such as a schema for `schema-check`, should fail fast.

## Troubleshooting

- If `schema-check` fails with `requires review_request.schema`, the caller did not populate the
  schema field before invoking the provider.
- If outcomes are too noisy, tighten the schema or rule set so the provider only checks the fields
  the runtime actually cares about.
- If a rule-based reviewer misses expected problems, verify that the referenced path exists in the
  subject and that numeric thresholds match the serialized value types.
- If `llm-critic` output fails validation, narrow the prompt, reduce optional fields, or reuse the
  deterministic reviewers where possible.
