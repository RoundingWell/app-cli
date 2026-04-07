## 1. Tests

- [x] 1.1 Add failing tests for `workspaces list` (success, empty, JSON output, API error)

## 2. Implementation

- [x] 2.1 Create `src/commands/workspaces.rs` with JSON:API structs, `WorkspaceRow`, `WorkspaceListOutput`, and `list` function
- [x] 2.2 Register `workspaces` module in `src/commands/mod.rs`
- [x] 2.3 Add `workspaces list` subcommand to `src/main.rs`

## 3. Docs

- [x] 3.1 Update README.md to document `workspaces list`

## 4. Verification

- [x] 4.1 Run `cargo clippy` and `cargo fmt`
- [x] 4.2 Run `cargo test` and confirm all tests pass
- [x] 4.3 Run `cargo build --release` to confirm clean build
