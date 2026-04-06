## Why

The `config default` subcommands (`set`, `get`, `rm`, `list`) ignore the global `--profile` flag and always operate on the configured default profile. This is inconsistent with all other commands (e.g. `auth`, `clinicians`) that correctly respect `--profile`.

## What Changes

- `config default set`, `get`, `rm`, and `list` will accept and use the `--profile` option to determine which profile to operate on, falling back to the configured default when `--profile` is not supplied.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `config-default`: Requirements must state that all subcommands respect the global `--profile` option.

## Impact

- `src/commands/config.rs`: `default_set`, `default_get`, `default_rm`, `default_list` functions and the `active_profile_name` helper must accept an optional profile override.
- `src/main.rs`: The `ConfigCommands::Default` dispatch block must pass `cli.profile.as_deref()` to each of those functions.
- `openspec/specs/config-default/spec.md`: Updated to document `--profile` behaviour.
- Tests for each affected function must cover the `--profile` override path.
