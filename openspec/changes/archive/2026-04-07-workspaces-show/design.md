## Context

The CLI currently supports `workspaces list` which calls `GET /workspaces` and returns id, slug, and name. No `GET /workspaces/:id` endpoint exists. The workspace resource includes a `settings` object in attributes (currently only `default_for_clinicians`, but could grow). The `roles show` command provides the nearest precedent: fetch list, match by UUID or name, display details.

## Goals / Non-Goals

**Goals:**
- Add `workspaces show <target>` that accepts a UUID or slug
- Reuse the existing `GET /workspaces` list call; match locally
- Display id, slug, name, and settings as a name/value table
- Support `--json` output

**Non-Goals:**
- Paging or filtering on the server side
- Modifying the workspace list response structure

## Decisions

**Match by UUID or slug**
- Detect UUID by parsing with the `uuid` crate (already a dependency)
- UUID matches on `id`; non-UUID string matches on `slug`
- Rationale: same pattern as `roles show` (UUID or name)

**Settings deserialization**
- Deserialize `attributes.settings` as `serde_json::Map<String, serde_json::Value>` so any current or future settings key is captured without code changes
- Plain output renders each key/value as a row in a markdown table sorted by key name
- JSON output includes the raw settings map under a `settings` key

**Output format**
Plain output:
```
ID:   <uuid>
Slug: <slug>
Name: <name>

| name                   | value |
|------------------------|-------|
| default_for_clinicians | true  |
```

JSON output:
```json
{"id": "...", "slug": "...", "name": "...", "settings": {"default_for_clinicians": true}}
```

**Code location**
- Add `show` function to `src/commands/workspaces.rs`
- Extend `WorkspaceAttributes` to include `settings: serde_json::Map<String, serde_json::Value>`
- Add `WorkspacesShowArgs` and `WorkspacesCommands::Show` to `src/cli.rs`
- Wire up in `src/main.rs`

## Risks / Trade-offs

- **Settings map grows**: Displaying all keys as-is means future settings are shown automatically; values are printed as their JSON representation (e.g., `true`, `"string"`, `42`). This is acceptable.
- **List is the only fetch path**: Fetching all workspaces to show one is O(n), fine for small workspace counts typical in this system.
