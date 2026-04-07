## 1. Move resolve_role to roles module

- [x] 1.1 Add `resolve_role` function to `src/commands/roles.rs` with `pub(crate)` visibility (identical signature and logic)
- [x] 1.2 Update `src/commands/clinicians.rs` to import and use `super::roles::resolve_role` instead of its local definition
- [x] 1.3 Remove the local `resolve_role` function from `src/commands/clinicians.rs`
- [x] 1.4 Run `cargo test` to confirm no regressions

## 2. Extend roles deserialization types

- [x] 2.1 Add `description` and `permissions` fields to `RoleAttributes` in `src/commands/roles.rs`

## 3. Implement roles show command

- [x] 3.1 Add `RoleShowOutput` struct with `id`, `name`, `label`, `description`, `permissions` fields; implement `CommandOutput` for plain display
- [x] 3.2 Add `pub async fn show(ctx, target, out)` to `src/commands/roles.rs` that calls `GET /roles`, finds the matching resource by UUID or name, and prints `RoleShowOutput`

## 4. Wire up CLI

- [x] 4.1 Add `RolesShowArgs` (with `target: String`) to `src/cli.rs`
- [x] 4.2 Add `Show(RolesShowArgs)` variant to `RolesCommands` enum in `src/cli.rs`
- [x] 4.3 Add `RolesCommands::Show` dispatch in `src/main.rs` (or wherever `RolesCommands` is matched)

## 5. Tests

- [x] 5.1 Write a failing test for `roles show` by UUID (mocked `GET /roles`)
- [x] 5.2 Write a failing test for `roles show` by name
- [x] 5.3 Write a failing test for target not found
- [x] 5.4 Write a failing test for API error on `GET /roles`
- [x] 5.5 Run `cargo test` to confirm all new tests pass

## 6. Docs and polish

- [x] 6.1 Update `README.md` to document `roles show <target>`
- [x] 6.2 Run `cargo clippy` and `cargo fmt`
