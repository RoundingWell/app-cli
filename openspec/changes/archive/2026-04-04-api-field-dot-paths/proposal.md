## Why

The `api --field` (`-f`) flag currently only supports flat key=value pairs, making it impossible to build nested JSON bodies like those required by the JSON:API format (e.g. `{attributes: {name: "..."}}`) without resorting to raw `--data` workarounds. Supporting dot-path keys enables users to construct nested request bodies directly from the CLI.

## What Changes

- `-f attributes.name="John Doe"` will produce `{"attributes": {"name": "John Doe"}}` in the request body
- `-f relationships.team.data.id=<uuid>` will produce deeply nested objects
- Multiple dot-path fields are merged into a single JSON object
- Flat keys (no dot) continue to work as before
- The request body is now built as a nested `serde_json::Value` instead of `HashMap<String, String>`

## Capabilities

### New Capabilities
- `api-field-dot-paths`: Support dot-notation in `-f`/`--field` arguments to produce nested JSON request bodies

### Modified Capabilities
<!-- No existing spec-level requirements are changing -->

## Impact

- `src/commands/api.rs`: `parse_field` and body-building logic updated
- Request body type changes from `HashMap<String, String>` to `serde_json::Value`
- No changes to CLI surface (`--field`/`-f` flag itself is unchanged)
- No breaking changes for existing flat key usage
