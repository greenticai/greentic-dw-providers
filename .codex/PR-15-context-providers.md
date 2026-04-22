# PR-15: Context Providers

## Title
feat(providers/context): add static, retrieval, and compressor context providers

## Dependencies
- greentic-dw PR-05
- workspace provider from PR-12 recommended

## Scope
Implement:
- `context/static`
- `context/retrieval`
- `context/compressor`

## Concrete work

### static
Assemble context from static prompt fragments + runtime metadata.

### retrieval
Load context fragments from:
- workspace artifacts
- memory references
- plan step references

### compressor
Apply deterministic truncation/summarization strategy:
- keep pinned fragments
- rank by score
- summarize overflow if summarizer configured
- otherwise truncate with explicit provenance notice

## Tests
- fragment ordering deterministic
- pinned fragments are never dropped
- overflow produces compression metadata
- retrieval preserves provenance refs

## Acceptance criteria
- Context packages are compiled documents and can be inspected in tests.
