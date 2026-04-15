# PR-10: Provider Family Scaffolds

## Title
feat(providers): scaffold planning, workspace, delegation, reflection, and context provider families

## Why
The providers repo already uses family-oriented organization with core + concrete implementations. The new
families should follow the same structure. citeturn154138view1

## Scope
Create empty family scaffolds and shared core crates:
```text
planning/core
workspace/core
delegation/core
reflection/core
context/core
```

## Concrete work
- add workspace members
- add top-level README entries
- add family-level docs
- add placeholder compile tests

## Acceptance criteria
- Workspace compiles with scaffold crates.
- CI includes the new crates.
