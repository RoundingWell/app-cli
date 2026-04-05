## Why

The `clinicians assign <role>` command uses the word "assign" which is ambiguous — it could imply ownership transfer. The word "grant" better reflects the intent of giving a clinician a role permission, and aligns with common RBAC (role-based access control) terminology.

## What Changes

- The `clinicians assign <role>` subcommand is renamed to `clinicians grant <role>`
- **BREAKING**: The `assign` subcommand is removed; users must use `grant` going forward

## Capabilities

### New Capabilities
- `clinicians-grant`: The `clinicians grant <role>` command grants a role to a clinician, replacing `clinicians assign`

### Modified Capabilities

## Impact

- `src/` code implementing the `clinicians assign` subcommand
- README.md and any docs referencing `clinicians assign`
- Tests covering the `assign` subcommand
