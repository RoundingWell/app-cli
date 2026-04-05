## Why

The error message for dot-path key conflicts references the full nested key (e.g., `foo.baz`) rather than the conflicting prefix segment (e.g., `foo`), making it harder to understand which top-level key caused the conflict. The spec requires the error to indicate the conflicting prefix, and the test assertion only loosely checks for `"foo"` rather than verifying the exact message wording.

## What Changes

- Fix `map_err` closure in `build_body` to extract the first segment of `k` (before the first dot) and use it as the conflicting prefix in the error message
- Update the test assertion to verify the exact error message wording rather than a loose `contains("foo")` check

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `api-field-dot-paths`: The error message for key path conflicts must reference the conflicting prefix segment, not the full dot-path key

## Impact

- `src/commands/api.rs`: `build_body` function — `map_err` closure on `dot_set`, and the test at line ~251
