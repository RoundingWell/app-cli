## 1. Rename CLI subcommand and internal function

- [x] 1.1 Rename the `Assign` variant and `AssignArgs` struct to `Grant` / `GrantArgs` in `src/cli.rs`
- [x] 1.2 Rename the `assign` function to `grant` in `src/commands/clinicians.rs`
- [x] 1.3 Update the dispatch match arm in `src/main.rs` from `assign` to `grant`

## 2. Update output messages and tests

- [x] 2.1 Update the success output message from "assigned to" to "granted" (or equivalent) in `src/commands/clinicians.rs`
- [x] 2.2 Rename all test functions and update test assertions that reference `assign` in `src/commands/clinicians.rs`

## 3. Update documentation

- [x] 3.1 Update README.md to replace all references to `clinicians assign` with `clinicians grant`
- [x] 3.2 Update any other docs (CONTRIBUTING.md, files in `docs/`) that reference `clinicians assign`

## 4. Verify

- [x] 4.1 Run `cargo test` and confirm all tests pass
- [x] 4.2 Run `cargo clippy` and `cargo fmt` and fix any issues
