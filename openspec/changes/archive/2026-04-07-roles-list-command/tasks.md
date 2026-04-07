## 1. Create the roles module

- [x] 1.1 Create `src/commands/roles.rs` with `RoleAttributes` (fields: `name`, `label`), `RoleResource`, `RoleListResponse`, `RoleRow`, and `RoleListOutput` types, modeled after `teams.rs`
- [x] 1.2 Add a `list` function in `src/commands/roles.rs` that calls `GET /roles`, maps to `RoleRow`, sorts by `label`, and prints via `Output`
- [x] 1.3 Register the new module in `src/commands/mod.rs` as `pub mod roles`

## 2. Update the clinicians module

- [x] 2.1 Remove `RoleAttributes`, `RoleResource`, and `RoleListResponse` from `src/commands/clinicians.rs`
- [x] 2.2 Update `resolve_role` in `src/commands/clinicians.rs` to use `roles::RoleListResponse` from the `roles` module
- [x] 2.3 Add `label` field to `RoleAttributes` in `src/commands/roles.rs` (if not already present) with `#[serde(default)]` to guard against missing field

## 3. Wire CLI

- [x] 3.1 Add `RolesArgs`, `RolesCommands`, and `RolesListArgs` structs to `src/cli.rs`, following the `TeamsArgs` pattern
- [x] 3.2 Add `Roles(RolesArgs)` variant to the top-level `Commands` enum in `src/cli.rs`
- [x] 3.3 Add dispatch for `Commands::Roles` in `src/main.rs` (or wherever CLI dispatch lives) calling `commands::roles::list`

## 4. Tests

- [x] 4.1 Write a test for `roles list` with multiple roles — verify sorted by `label`
- [x] 4.2 Write a test for `roles list` with an empty response
- [x] 4.3 Write a test for `roles list` with an API error response
- [x] 4.4 Verify existing `clinicians grant` / `clinicians prepare` tests still pass after role type move

## 5. Docs and polish

- [x] 5.1 Update `README.md` to document the `roles list` command
- [x] 5.2 Run `cargo clippy` and fix any warnings
- [x] 5.3 Run `cargo fmt`
- [x] 5.4 Run `cargo test` and confirm all tests pass
