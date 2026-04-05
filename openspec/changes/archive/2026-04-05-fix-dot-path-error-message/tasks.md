## 1. Tests

- [x] 1.1 Update `test_build_body_key_conflict_error` to assert the error message is `"Unable to set field foo.baz because it conflicts with another field"`
- [x] 1.2 Add `test_build_body_key_conflict_error_multilevel` that passes `-f a.b=x -f a.b.c=y` and asserts the error message is `"Unable to set field a.b.c because it conflicts with another field"`
- [x] 1.3 Run `cargo test test_build_body_key_conflict_error` to confirm both tests fail

## 2. Implementation

- [x] 2.1 In `build_body` (`src/commands/api.rs`), replace the `map_err` closure with `|_| anyhow::anyhow!("Unable to set field {} because it conflicts with another field", k)`
- [x] 2.2 Run `cargo test` to confirm all tests pass

## 3. Polish

- [x] 3.1 Run `cargo clippy` and resolve any warnings
- [x] 3.2 Run `cargo fmt`
