## 1. Fix Error Message

- [x] 1.1 In `build_body` in `src/commands/api.rs`, update the `map_err` closure on `dot_set` to extract the first segment of `k` via `k.split('.').next().unwrap_or(&k)` and use it as the prefix in the error message: `"field key conflict: '{}' is both a leaf and a nested path"`

## 2. Update Test

- [x] 2.1 In `test_build_body_key_conflict_error`, replace the loose `msg.contains("foo")` assertion with an exact match: assert the error message equals `"field key conflict: 'foo' is both a leaf and a nested path"`

## 3. Verify

- [x] 3.1 Run `cargo test test_build_body_key_conflict_error` to confirm the test passes
- [x] 3.2 Run `cargo clippy` and `cargo fmt` to ensure no lint or formatting issues
