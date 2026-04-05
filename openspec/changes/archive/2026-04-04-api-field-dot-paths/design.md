## Context

The `api` command's `-f`/`--field` flag accepts `key=value` pairs and builds a flat JSON body (`HashMap<String, String>`). The RoundingWell API uses JSON:API format, which requires nested structures like `{"data": {"type": "...", "attributes": {"name": "..."}}}`. Users cannot currently express these nested bodies with `-f` alone.

The change is confined to `src/commands/api.rs` — specifically the `parse_field` function and the body-building block in `run`.

## Goals / Non-Goals

**Goals:**
- Parse dot-separated keys (e.g. `attributes.name`) into nested JSON objects
- Merge multiple dot-path fields into a single `serde_json::Value` body
- Preserve backward compatibility: flat keys without dots work identically to today
- Values remain strings (no type coercion)

**Non-Goals:**
- Array index notation (e.g. `data[0].id`) — out of scope
- Value type coercion (booleans, numbers) — values stay as JSON strings
- Changes to any other CLI flag or command

## Decisions

### Use `serde_json::Value` for body construction (instead of `HashMap<String, String>`)

**Decision:** Replace `HashMap<String, String>` with `serde_json::Value::Object` for the request body.

**Rationale:** `serde_json::Value` is the natural representation for an arbitrarily nested JSON document. The `reqwest` client already serialises it correctly via `.json(...)`. No new dependencies are required — `serde_json` is already in the dependency graph.

**Alternative considered:** Keep `HashMap` and post-process into a nested structure. Rejected because it adds a conversion step with no benefit.

### Use `json_dotpath` crate for dot-path insertion

**Decision:** Add the `json_dotpath` crate and use its `dot_set` method to insert values at dot-separated paths into a `serde_json::Value`.

**Rationale:** `json_dotpath` provides a well-tested, idiomatic implementation of dot-path traversal and insertion on `serde_json::Value`, covering merging of shared prefixes and error handling for path conflicts. Using it avoids hand-rolling traversal logic and reduces the risk of edge-case bugs.

**Alternative considered:** Implement a custom recursive `insert_dot_path` helper. Rejected in favour of the maintained library solution.

### Recursive merge for overlapping dot-path prefixes

**Decision:** When two fields share a key prefix (e.g. `attributes.name` and `attributes.age`), merge them into the same nested object rather than overwriting.

**Rationale:** This is the expected behaviour — users will routinely supply multiple `attributes.*` fields in a single command.

**Implementation:** Call `dot_set` from `json_dotpath` for each field in sequence; the crate handles descent and merging automatically.

### Integer path segments produce JSON arrays (via `dot_set`)

**Decision:** Bare-integer segments (e.g. `data.0.id`) are treated as array offsets by `json_dotpath::dot_set`, producing `serde_json::Value::Array` nodes — not object keys with integer string names.

**Rationale:** This is the behavior provided by the `json_dotpath` crate and requires no additional implementation work. It is genuinely useful for JSON:API payloads that include array relationships (e.g. `relationships.workspaces.data.0.id`). Suppressing it would require replacing or wrapping `dot_set` with no benefit.

**Note:** The original design stated "dot-paths are object-only" and that integer segments are treated as string keys. This was incorrect — it described intended behavior that was never implemented; the crate always produced arrays for integer segments.

## Risks / Trade-offs

- **Key collision:** If a user supplies both `foo=bar` and `foo.baz=qux` the second insertion would fail to descend into a string value. → Mitigation: return a clear error message: `"field key conflict: 'foo' is both a leaf and a nested path"`.
- **Dot in value:** Values may contain dots without issue — only the key portion (left of `=`) is split on `.`.
- **No migration needed:** This is an additive, backward-compatible CLI change with no persistent state.

## Open Questions

- ~~Should numeric-only path segments (e.g. `items.0.id`) emit a warning rather than silently treat the segment as a string key?~~ **Resolved:** Integer segments produce JSON arrays via `dot_set` — this is documented as supported behavior. No warning is needed.
