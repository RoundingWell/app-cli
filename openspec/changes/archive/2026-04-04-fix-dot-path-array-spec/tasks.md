## 1. Correct Archived Design Document

- [x] 1.1 In `openspec/changes/archive/2026-04-04-api-field-dot-paths/design.md`, update the "No array support" Decision section to state that bare-integer segments are treated as array offsets by `dot_set`, not as string object keys
- [x] 1.2 Update the Open Questions note in the same file to reflect that numeric segments are now documented as array-producing behavior

## 2. Update Spec

- [x] 2.1 Archive the delta spec into `openspec/specs/api-field-dot-paths/spec.md` by appending the new integer-segment requirement and scenarios from the change spec

## 3. Tests

- [x] 3.1 In `src/commands/api.rs`, add a test `test_field_integer_segment_creates_array` that passes `-f items.0.id=abc` and asserts the body is `{"items": [{"id": "abc"}]}`
- [x] 3.2 Add a test `test_field_multiple_integer_segments` that passes `-f items.0.id=abc -f items.1.id=def` and asserts `{"items": [{"id": "abc"}, {"id": "def"}]}`
- [x] 3.3 Add a test `test_field_integer_segment_nested` covering `relationships.workspaces.data.0.type=workspaces -f relationships.workspaces.data.0.id=<uuid>` and assert the expected nested array structure

## 4. Verify

- [x] 4.1 Run `cargo test` — all tests pass
- [x] 4.2 Run `cargo clippy` — no warnings
- [x] 4.3 Run `cargo fmt` — no formatting changes
