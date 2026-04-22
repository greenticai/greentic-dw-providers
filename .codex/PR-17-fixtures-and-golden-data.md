# PR-17: Fixtures and Golden Data

## Title
test(providers): add fixtures and golden outputs for planning/context/workspace/reflection/delegation

## Scope
Add:
- plan fixtures
- context fixtures
- artifact fixtures
- delegation decision fixtures
- reflection fixtures

Store deterministic golden outputs under:
```text
tests/golden/
```

## Acceptance criteria
- Golden files make provider regressions obvious in code review.
