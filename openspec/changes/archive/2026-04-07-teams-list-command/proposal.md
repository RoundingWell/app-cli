## Why

The CLI currently has no way to list teams, making it difficult for users to discover team IDs and abbreviations needed for other commands. Adding a `teams list` command provides a quick reference for available teams.

## What Changes

- Add new `teams list` subcommand under the `teams` command group
- Fetch all teams from the API and display a sorted table with `id`, `abbr`, and `name` columns
- Table is sorted by `abbr` for easy scanning

## Capabilities

### New Capabilities

- `teams-list`: Fetch and display a table of all teams with id, abbr, and name, sorted by abbr

### Modified Capabilities

<!-- None -->

## Impact

- New `src/commands/teams/` module (or added to existing teams command structure)
- New API call to the teams endpoint
- Output formatted as a table consistent with other list commands
