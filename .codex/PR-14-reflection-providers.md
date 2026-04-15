# PR-14: Reflection Providers

## Title
feat(providers/reflection): add schema-check, rules, and llm-critic reflection providers

## Dependencies
- greentic-dw PR-04

## Scope
Implement three providers:

### schema-check
Verifies output against declared schema.

### rules
Applies deterministic rules over artifacts / step outputs.

### llm-critic
Uses an LLM to generate a typed `ReviewOutcome`, not free text.

## Concrete work
- shared review helper utilities
- rule DSL kept intentionally minimal:
  - exists
  - contains
  - count_gte
  - score_gte
  - schema_valid

## Tests
- schema-check accepts valid output and rejects invalid output
- rules provider returns `Revise` with findings
- llm-critic output is validated strictly

## Acceptance criteria
- Cheap deterministic reflection exists before expensive LLM reflection.
