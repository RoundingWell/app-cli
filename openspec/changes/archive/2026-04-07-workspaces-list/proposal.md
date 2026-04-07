## Why

The CLI lacks workspace management commands, making it difficult to browse workspace resources. Adding `workspaces list` brings workspace visibility in line with existing `teams` and `roles` commands.

## What Changes

- Add `workspaces list` command: fetches and displays all workspaces with `id`, `slug`, and `name` columns

## Capabilities

### New Capabilities

- `workspaces-list`: List all workspaces with id, slug, and name

### Modified Capabilities

## Impact

- New `workspaces` subcommand group added to the CLI
- New API call to the workspaces endpoint (JSON:API format)
- `src/commands/workspaces.rs` (new file) and registration in `src/main.rs`
