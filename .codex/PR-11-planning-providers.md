# PR-11: Planning Providers

## Title
feat(providers/planning): add static-plan and llm-outline planning providers

## Dependencies
- greentic-dw PR-01

## Scope
Implement two planning providers:

### A) `planning/static`
Use a predefined graph template supplied in config.

### B) `planning/llm-outline`
Call an LLM, but require it to return a valid `PlanDocument` only.
No execution, no hidden side effects.

## File tree
```text
planning/
  static/
    Cargo.toml
    src/lib.rs
    src/config.rs
    src/provider.rs
    tests/static_plan.rs
  llm-outline/
    Cargo.toml
    src/lib.rs
    src/config.rs
    src/provider.rs
    src/prompt.rs
    tests/llm_outline.rs
```

## Concrete work

### static provider
Config:
- embedded plan fixture
- optional variable interpolation
- strict validation on startup

### llm-outline provider
Config:
- llm provider ref
- max_steps
- allowed_step_kinds
- schema strictness
- replan_prompt_variant

The provider must:
- build a prompt requesting only plan JSON
- validate returned graph
- reject invalid output
- optionally retry once with validator feedback

## Tests
- static returns exact expected plan
- llm-outline rejects malformed graph
- llm-outline preserves deterministic validation behavior

## Acceptance criteria
- At least one fully deterministic planner exists.
- One LLM-backed planner exists with strict structured output.
