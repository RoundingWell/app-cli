## Context

The CLI has established patterns for listing resources (e.g., `teams list`, `roles list`). Workspaces are a core resource exposed by the API. This change adds `workspaces list` following those same patterns.

## Goals / Non-Goals

**Goals:**
- Add `workspaces list` following the same pattern as `teams list`
- Maintain consistency with existing command conventions (JSON:API, `--json` flag, markdown table output)

**Non-Goals:**
- Showing, creating, updating, or deleting workspaces
- Filtering or searching workspaces

## Decisions

**Follow `teams.rs` pattern exactly**
New file `src/commands/workspaces.rs` with JSON:API deserialization structs, a `CommandOutput` impl, and an async `list` function. Rationale: consistency, no new dependencies needed.

**Sort `workspaces list` by `name`**
No natural short identifier like `abbr` exists. Sorting by `name` is consistent with user expectations.

## Risks / Trade-offs

- Minimal risk; this is a straightforward read-only list command following an established pattern.
