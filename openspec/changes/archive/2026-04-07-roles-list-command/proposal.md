## Why

The CLI currently has no way to list roles, which are needed for commands like `clinicians grant`. Adding a `roles list` command provides discoverability and enables scripting against role data without relying on hardcoded values.

## What Changes

- Add a new `roles` module with a `roles list` command
- Move role type definitions from the `clinicians` module to the new `roles` module
- Output includes `id`, `name`, and `label` attributes, sorted by `label`

## Capabilities

### New Capabilities

- `roles-list`: List all roles with id, name, and label, sorted by label

### Modified Capabilities

- `clinicians-grant`: Role types move to the `roles` module; import path changes but behavior is unchanged

## Impact

- New `src/roles/` module
- `src/clinicians/` module updated to import role types from `src/roles/`
- New API endpoint: `GET /roles`
- README.md updated with `roles list` command documentation
