## Context

The CLI has established patterns for listing resources (`teams list`, `workspaces list`). Artifacts are a queryable resource that require type, path, and term filters on every request — the API does not support unfiltered listing.

## Goals / Non-Goals

**Goals:**
- Add `artifacts list <type> --path=<path> --term=<term>` following the same patterns as existing list commands
- Pass all three filters as query parameters to `GET /artifacts`
- Display results as a markdown table with `artifact`, `identifier`, `values` columns

**Non-Goals:**
- Creating, updating, or deleting artifacts
- Pagination or cursor-based listing
- Showing a single artifact by ID

## Decisions

**Follow `workspaces.rs` pattern exactly**
New file `src/commands/artifacts.rs` with JSON:API deserialization structs, a `CommandOutput` impl, and an async `list` function. No new dependencies needed.

**`--path` and `--term` as required named options, `<type>` as positional argument**
The issue specifies the signature `artifacts list <type> --path=<path> --term=<term>`. Using clap `#[arg(required = true)]` for the options enforces the requirement at parse time with a clear error message.

**Deserialize `values` as `serde_json::Map<String, serde_json::Value>`**
The `values` attribute is a `map<string, mixed>`. Using `serde_json::Map<String, serde_json::Value>` models this exactly. For table display, serialize the whole map to a compact JSON string via `.to_string()`.

## Risks / Trade-offs

- Minimal risk; read-only command following an established pattern.
- `values` serialized as a compact JSON string may be wide for large maps, but is correct without needing to enumerate all possible keys.
