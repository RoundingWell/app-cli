## Why

Clinician data sometimes needs to be corrected or updated after initial creation, but the CLI currently has no way to do this, requiring users to fall back to the web UI or direct API calls. Adding an `update` subcommand to `clinicians` closes this gap for common fields.

## What Changes

- New subcommand: `rw clinicians update <target> --field <name> --value <val>`
- `<target>` accepts `"me"` (authenticated user), an email address, or a UUID
- Updatable fields: `name`, `email`, `npi`, `credentials`
- Field-specific validation before sending to the API:
  - `name`: non-empty string
  - `email`: valid email format
  - `npi`: string or empty/null (nullable)
  - `credentials`: one or more credential strings (e.g. `"RN"`)

## Capabilities

### New Capabilities

- `clinicians-update`: Update a clinician's attributes via the CLI, with target resolution and per-field validation

### Modified Capabilities

<!-- none -->

## Impact

- Adds a new `update` subcommand under the existing `clinicians` command module
- Makes a `PATCH` request to the clinicians API endpoint in JSON:API format
- Requires target resolution logic (me / email / UUID) shared with existing clinician lookup patterns
- No new dependencies expected
