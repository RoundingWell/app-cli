## 1. Tests

- [x] 1.1 Add failing unit test for `parse_field` with a dot-path key (e.g. `attributes.name=John`)
- [x] 1.2 Add failing unit test for multiple dot-path fields sharing a prefix (merged into one object)
- [x] 1.3 Add failing unit test for deeply nested dot-path (3+ levels)
- [x] 1.4 Add failing unit test for mixed flat and dot-path fields
- [x] 1.5 Add failing unit test for dot in value (value not split)
- [x] 1.6 Add failing unit test for key conflict error (`foo=bar` then `foo.baz=qux`)

## 2. Core Implementation

- [x] 2.1 Add `json_dotpath` to `Cargo.toml` dependencies
- [x] 2.2 Replace `HashMap<String, String>` body with `serde_json::Value` in `api.rs`
- [x] 2.3 Update the body-building loop in `run` to use `dot_set` from `json_dotpath` for each field
- [x] 2.4 Return a descriptive error when `dot_set` fails (e.g. path conflict)

## 3. Verification

- [x] 3.1 Run `cargo test` — all tests pass
- [x] 3.2 Run `cargo clippy` — no warnings
- [x] 3.3 Run `cargo fmt` — no formatting changes
- [x] 3.4 Manually smoke-test: `rw api <endpoint> -f attributes.name="Test"` sends correct nested body (manual step)

## 4. Documentation

- [x] 4.1 Update `README.md` (or relevant docs) to document dot-path syntax in the `api` command section
