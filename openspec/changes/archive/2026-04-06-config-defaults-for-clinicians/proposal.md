## Why

The `clinicians prepare` command hard-codes `role=employee` and `team=NUR`, making it impossible to use the CLI against environments or accounts that use different roles or team codes without modifying source code. Users need to be able to configure sensible default per profile.

## What Changes

- Add `config default set <key> <value>` command to write a default value into the active profile config
- Add `config default get <key>` command to read a default value (returns `null` if unset)
- Add `config default rm <key>` command to remove a default value (no-ops if unset)
- Restrict allowed keys to `team` and `role`
- Update `clinicians prepare` to read `default.role` and `default.team` from config, falling back to `employee` and `NUR` respectively when unset

## Capabilities

### New Capabilities

- `config-default`: Manage per-profile default key/value pairs (`team`, `role`) via `config default set/get/rm` subcommands

### Modified Capabilities

- `clinicians-register`: `clinicians prepare` now reads role and team from config default instead of hard-coded values (fallback behavior preserved)

## Impact

- `src/commands/config.rs` (or equivalent): new `default` subcommand with `set`, `get`, `rm` actions
- Profile config file: new `[default]` section (or equivalent map) stored alongside existing profile data
- `src/commands/clinicians.rs` (or equivalent): `prepare` subcommand reads config default before applying hard-coded fallbacks
- No API changes; purely CLI/config-layer change
- No breaking changes to existing commands or config files
