## Why

The `clinicians` command currently supports listing and mutating clinician records, but there is no way to fetch and display details for a single clinician. Adding `clinicians show` gives users a direct way to inspect any clinician by UUID, email, or their own identity.

## What Changes

- Add `rw clinicians show <target>` command where `<target>` is a UUID, email address, or the literal string `"me"`
- When `<target>` is `me`, call `GET /clinicians/me`
- When `<target>` is a UUID, call `GET /clinicians/:id`
- When `<target>` is an email, resolve it via `GET /clinicians?filter[email]={email}` and then display the result

## Capabilities

### New Capabilities
- `clinicians-show`: Fetch and display a single clinician by UUID, email, or `me`

### Modified Capabilities

## Impact

- `src/commands/clinicians/` — new subcommand added
- `GET /clinicians/:id` and `GET /clinicians/me` API endpoints consumed
- `GET /clinicians?filter[email]={email}` used for email resolution
