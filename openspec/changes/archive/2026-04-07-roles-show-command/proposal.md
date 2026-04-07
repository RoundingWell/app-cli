## Why

The CLI needs a way to inspect a single role by UUID or name, including its permissions. The `resolve_role` function is currently in clinicians but belongs in roles for better cohesion.

## What Changes

- Add `roles show <target>` command that accepts a UUID or role name
- Display role UUID, name, label, description, and list of permissions
- Move `resolve_role` function from the clinicians module to the roles module

## Capabilities

### New Capabilities

- `roles-show`: Look up and display a single role by UUID or name, including its permissions

### Modified Capabilities

- `roles-list`: No requirement changes; `resolve_role` relocation is an implementation detail only

## Impact

- New `roles show` subcommand added to the `roles` command group
- `resolve_role` function moved from `src/clinicians.rs` (or equivalent) to `src/roles.rs`
- Clinicians commands that use `resolve_role` must import it from roles
- Requires a new API call to fetch a single role by ID (after resolving name → UUID via list)
