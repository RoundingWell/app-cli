## Why

The `rw` CLI can list teams but cannot display the details of a single team. Adding `teams show` brings teams to parity with `roles show` and `workspaces show`, giving users a quick way to inspect a specific team by id or abbreviation.

## What Changes

- Add `rw teams show <target>` command
- `<target>` matches on team id or abbreviation (`abbr`)
- Uses list + client-side match (no `GET /teams/:id` endpoint exists)

## Capabilities

### New Capabilities

- `teams-show`: Display details for a single team matched by id or abbreviation

### Modified Capabilities

<!-- none -->

## Impact

- New subcommand under the existing `teams` command group
- Reuses the existing teams list API call (`GET /teams`)
- No new API dependencies
