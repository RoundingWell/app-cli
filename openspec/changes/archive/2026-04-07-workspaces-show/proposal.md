## Why

Users need to inspect a single workspace's details — including its settings — without manually parsing list output. The `workspaces show` command provides a direct, readable view of a workspace by UUID or slug.

## What Changes

- Add `workspaces show <target>` subcommand that accepts a UUID or slug
- Fetch workspaces via the list endpoint, then match locally (no dedicated show endpoint exists)
- Display workspace id, slug, name, and a settings table (name, value)

## Capabilities

### New Capabilities
- `workspaces-show`: Show a single workspace by UUID or slug, displaying id, slug, name, and a settings table

### Modified Capabilities

## Impact

- `src/commands/workspaces/` — new `show` subcommand module
- Reuses existing workspace list API call and JSON:API parsing
