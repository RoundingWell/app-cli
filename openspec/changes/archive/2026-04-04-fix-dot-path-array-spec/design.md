## Context

The `api-field-dot-paths` feature uses `json_dotpath::dot_set` to insert values at dot-separated paths into a `serde_json::Value`. The archived design document stated that bare-integer path segments are treated as literal string object keys. This is factually incorrect: `dot_set` interprets bare integers as array offsets and creates `serde_json::Value::Array` nodes. The spec was silent on this behavior, leaving it undocumented and untested.

## Goals / Non-Goals

**Goals:**
- Correct the archived design document to describe integer-segment behavior accurately
- Add spec requirements and scenarios for numeric path segments
- Add tests that exercise the actual array behavior produced by `dot_set`

**Non-Goals:**
- Changing the implementation — `dot_set` already produces arrays for integer segments and that behavior is correct
- Supporting richer array syntax (e.g. `push`, `append`, negative indices)

## Decisions

### Accept array behavior from `dot_set` as the intended behavior

**Decision:** Treat integer path segments as array offsets (the behavior `dot_set` already provides) and document it as a supported capability.

**Rationale:** The behavior is already present in the implementation; suppressing it would require wrapping or replacing `dot_set`. Accepting and documenting it is simpler and more useful — users can express JSON:API `relationships.team.data.0.id` patterns.

**Alternative considered:** Detect bare-integer segments and return an error, treating array creation as unsupported. Rejected because the underlying library already handles it correctly and the capability is genuinely useful.

### Correct archived design in-place

**Decision:** Update the "No array support" section of the archived design document directly.

**Rationale:** The archive is the authoritative record of design decisions for that change. A comment in-place is clearer than a separate errata document.

## Risks / Trade-offs

- **Sparse arrays:** `dot_set` will create sparse arrays if the user skips indices (e.g. only sets index `2`). Intermediate slots will be `null`. This is unlikely in practice and matches standard JSON array semantics. → No mitigation needed now; document if users encounter confusion.
- **Conflict with object keys:** If a user first sets `foo.bar=x` (creating an object at `foo`) then sets `foo.0=y`, `dot_set` will error because it cannot index an object with an integer. This is already covered by the existing key-conflict error requirement.
