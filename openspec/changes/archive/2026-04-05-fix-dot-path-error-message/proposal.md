## Why

When `build_body` encounters a dot-path key conflict (e.g. `-f a.b=x` followed by `-f a.b.c=y`), the custom `map_err` closure constructs an error message referencing a "leaf" and "nested path" relationship. This message is both inaccurate for multi-level conflicts (it always reports the first segment, not the actual conflicting node) and unnecessarily specific—the underlying `dot_set` call already returns a clear error. The original spec that mandated this custom message was wrong.

## What Changes

- Replace the custom error message in `build_body`'s `map_err` closure with `"Unable to set field {key} because it conflicts with another field"`, using the full dot-path key.
- Update `test_build_body_key_conflict_error` to assert the new message.
- Add a test for a multi-level conflict to confirm the correct key appears in the message.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `api-field-dot-paths`: Key conflict error message is `"Unable to set field {key} because it conflicts with another field"`, using the full dot-path key. No longer references a prefix segment or leaf/nested-path relationship.

## Impact

- `src/commands/api.rs`: `build_body` function and `test_build_body_key_conflict_error` test.
- No API, CLI interface, or dependency changes.
