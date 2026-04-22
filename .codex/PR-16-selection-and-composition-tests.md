# PR-16: Provider Selection and Composition Tests

## Title
test(providers): add composition tests covering valid and invalid deep-agent provider combinations

## Scope
Add integration tests proving provider combinations compose correctly.

## Test matrix
- planning + context only
- planning + context + workspace
- planning + context + reflection
- full stack all five families
- delegate step without delegation provider -> rejected
- mandatory reflection policy without reflection provider -> rejected

## Acceptance criteria
- Provider composition failures are explained clearly.
