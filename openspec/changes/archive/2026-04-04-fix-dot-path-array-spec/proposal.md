## Why

The archived design for `api-field-dot-paths` incorrectly states that bare-integer path segments (e.g. `data.0.id`) are treated as literal string object keys. In reality, `json_dotpath::dot_set` treats bare integers as array offsets, producing JSON arrays — a meaningful behavioral difference that is unspecified and untested.

## What Changes

- Correct the `api-field-dot-paths` design document to accurately describe integer-segment behavior (array offsets, not string keys)
- Add a spec requirement and scenarios covering numeric path segments (array creation/indexing via `dot_set`)
- Add tests that confirm the actual array behavior

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `api-field-dot-paths`: Add requirement covering numeric path segments — integers in a dot-path create JSON arrays, not object keys with integer string names

## Impact

- `openspec/changes/archive/2026-04-04-api-field-dot-paths/design.md` — correct the "No array support" section
- `openspec/specs/api-field-dot-paths/spec.md` — add requirement + scenarios for integer segment behavior
- `src/commands/api.rs` — add tests; no behavior change expected (implementation is already correct)
