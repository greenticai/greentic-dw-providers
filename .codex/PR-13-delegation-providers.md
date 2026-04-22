# PR-13: Delegation Providers

## Title
feat(providers/delegation): add static-router and capability-match delegation providers

## Dependencies
- greentic-dw PR-03

## Scope
Implement:
- `delegation/static-router`
- `delegation/capability-match`

## Concrete work

### static-router
Config file maps:
- step kind
- schema
- tags
- goal patterns
to target agent ids

### capability-match
Use manifest-declared agent capabilities and expected output schema to choose a target.

Must support:
- none
- single
- parallel

## Safety rules
- never delegate to an unavailable agent
- never route around control policy
- return rationale list for audit

## Tests
- deterministic routing for identical inputs
- parallel fanout yields explicit target list
- missing agent returns clean error
- control-denied route remains denied

## Acceptance criteria
- Subtask routing exists without hidden runtime shortcuts.
