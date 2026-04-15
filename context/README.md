# Context Providers

Context providers assemble the input package presented to later planning, execution, or reflection
steps.

The shared contract uses the `dw.context.*` provider namespace and the `cap://dw.context`
capability family.

This family currently includes:

- `core` for context fragments and assembled context packages
- `static` for deterministic assembly from fixed fragments and runtime metadata
- `retrieval` for provenance-aware loading from workspace artifacts, plan steps, and memory refs
- `compressor` for deterministic ranking, truncation, and overflow summarization metadata

Concrete providers share one normalized context fragment model with score, pinning, and provenance.

## Overview

Use the context family when a runtime needs to assemble, rank, and compress the information that a
later step should see. Every backend shares the contract from `context/core`:

- `ContextRequest` carries a request id, optional hints, and runtime metadata
- `ContextFragment` carries structured content plus provenance, source kind, score, and pinning
- `ContextPackage` carries ordered fragments and assembly metadata

Choose a provider by role in the pipeline:

- `static` for baseline prompt fragments and runtime metadata injection
- `retrieval` for loading fragments from workspace history, plan steps, or memory refs
- `compressor` for deterministic ranking, truncation, and overflow summarization

## Config Reference

Shared runtime inputs:

- `request_id`: stable assembly id
- `hints`: optional assembly hints for future backends
- `runtime_metadata`: structured metadata that can be materialized into the package

Provider-specific configuration:

- `context/static`
  - ordered `fragments`
  - `include_runtime_metadata`: append runtime metadata as a pinned fragment when present
- `context/retrieval`
  - injected workspace provider
  - ordered `refs` made of `Workspace`, `PlanStep`, or `MemoryRef` sources
  - each ref carries an explicit score
- `context/compressor`
  - `max_fragments`: cap on returned fragment count before summary handling
  - `summarizer_configured`: whether overflow becomes a pinned summary fragment instead of silent
    truncation

## Example Manifest Snippet

```json
{
  "providers": [
    {
      "id": "context-static",
      "type": "dw.context.static",
      "capabilities": ["cap://dw.context.assemble"],
      "component": {
        "ref": "oci://ghcr.io/greenticai/packs/dw/context/static-pack:0.5.4"
      },
      "config": {
        "include_runtime_metadata": true
      }
    }
  ]
}
```

## Expected Runtime Behavior

- Context assembly is usually compositional: static fragments first, retrieved fragments next,
  compression last.
- Retrieval providers should attach provenance so later steps can explain where each fragment came
  from.
- Compressor providers keep pinned fragments even when lower-scored than dropped content.
- Returned package metadata should make truncation obvious, especially when overflow is summarized.
- For deterministic backends, identical inputs should produce identical fragment ordering.

## Troubleshooting

- If important content disappears during compression, check whether it was pinned and whether its
  score is high enough relative to competing fragments.
- If a workspace retrieval ref yields nothing, confirm the artifact exists in the requested scope
  and has at least one stored version.
- If plan-step retrieval yields nothing, the requested `step_id` is probably missing from the plan
  document passed into retrieval.
- If runtime metadata is not present in the final package, verify that `include_runtime_metadata`
  is enabled and the request actually carries non-empty metadata.
