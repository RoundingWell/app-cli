## Context

The `build_body` function in `src/commands/api.rs` uses `json_dotpath::dot_set` to insert dot-path fields into a `serde_json::Value`. When a conflict occurs (e.g., `foo=bar` then `foo.baz=qux`), the `map_err` closure constructs an error message using `k` — the full key being set at the time of failure (e.g., `foo.baz`). However, the conflict is with the already-set prefix (`foo`), not the descending key. The existing design doc specifies the message should read: `"field key conflict: 'foo' is both a leaf and a nested path"`, where `foo` is the conflicting prefix.

Additionally, the test at `test_build_body_key_conflict_error` only asserts `msg.contains("foo")`, which passes even if the message references the wrong key segment.

## Goals / Non-Goals

**Goals:**
- Error message references the conflicting prefix (first dot-path segment that collides), not the full descending key
- Test asserts the exact error message wording

**Non-Goals:**
- Changes to any other error message or behavior
- Handling multi-level prefix conflicts differently

## Decisions

### Extract the first segment of `k` as the conflict prefix

**Decision:** In the `map_err` closure, split `k` on `'.'` and take the first segment as the conflicting prefix for the error message.

**Rationale:** The conflict always occurs at the root segment that was previously set as a leaf. Splitting on `'.'` and taking `next()` is a zero-allocation, infallible operation that correctly identifies the prefix. This matches the wording specified in the original design doc.

**Implementation:**
```rust
let prefix = k.split('.').next().unwrap_or(&k);
anyhow::anyhow!("field key conflict: '{}' is both a leaf and a nested path", prefix)
```

**Alternative considered:** Include both the prefix and the full key (e.g., `"'foo' (from 'foo.baz') is both a leaf and a nested path"`). Rejected — the extra detail adds noise without aiding the user; the prefix alone is sufficient to identify the problem.

### Pin the test assertion to exact message wording

**Decision:** Replace `msg.contains("foo")` with an exact string match on the expected error message.

**Rationale:** A loose `contains` check let the bug go undetected — the message contained `"foo"` as part of `"foo.baz"`. An exact assertion catches future regressions.

## Risks / Trade-offs

- **Flat key conflict** (no dot in `k`): `split('.').next()` returns the whole key, so the message is still correct for keys with no dot separator.
