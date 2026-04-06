## Why

The `config default` subcommand supports `set`, `get`, and `rm` operations but provides no way to view all currently configured defaults at once. Users must query each key individually to inspect their active profile's defaults.

## What Changes

- Add a new `config default list` subcommand that prints all defined default key/value pairs from the active profile.

## Capabilities

### New Capabilities

- `config-default-list`: List all defined default key/value pairs in the active profile config.

### Modified Capabilities

- `config-defaults`: Add the `list` subcommand requirement to the existing spec.

## Impact

- `src/commands/config.rs`: Add `list` variant to the `ConfigDefault` subcommand enum and implement the handler.
- `src/config.rs`: No structural changes expected; reads from existing `default` map.
- `README.md`: Document the new subcommand.
