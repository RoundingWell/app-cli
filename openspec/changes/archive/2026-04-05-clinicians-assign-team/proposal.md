## Why

Clinicians need to be assignable to teams from the CLI, but currently only role-granting is supported. Adding team assignment with flexible lookup (uuid, abbr, or name) enables operators to manage team membership without needing exact UUIDs.

## What Changes

- Add a new `clinicians assign <target> <team>` subcommand
- The `<team>` argument resolves a team by UUID, abbreviated name (case-insensitive), or full name (case-insensitive)
- The `<target>` clinician argument follows the same resolution pattern already used by other `clinicians` subcommands

## Capabilities

### New Capabilities

- `clinicians-assign-team`: Assign a clinician to a team by UUID, abbr, or name (case-insensitive)

### Modified Capabilities

<!-- None -->

## Impact

- New subcommand added under the `clinicians` command group
- Requires a new API call to fetch teams list for abbr/name resolution
- Follows the same pattern as `clinicians grant` for clinician resolution and API interaction
