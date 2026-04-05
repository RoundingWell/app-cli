## Why

There is currently no way to create a new clinician via the CLI. Adding a `clinicians register` command fills this gap, enabling operators to onboard clinicians without leaving the terminal.

## What Changes

- Add `clinicians register <email> <name>` subcommand that issues a `POST /clinicians` request
- Support `--role` option to set the clinician's role (resolved by UUID or name via `resolve_role`)
- Support `--team` option to assign a team at registration time (resolved by UUID, abbreviation, or full name via `resolve_team`)

## Capabilities

### New Capabilities

- `clinicians-register`: Register a new clinician via the API, with optional role and team assignment

### Modified Capabilities

<!-- No existing spec-level behavior changes -->

## Impact

- `src/cli.rs`: Add `Register` variant to clinicians subcommand enum
- `src/commands/clinicians.rs`: Add `register` handler function
- `README.md` / docs: Document the new subcommand
