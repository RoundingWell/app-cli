## 1. Tests

- [x] 1.1 Add failing test for `show` by UUID (plain output)
- [x] 1.2 Add failing test for `show` by slug (plain output)
- [x] 1.3 Add failing test for `show` with `--json` output
- [x] 1.4 Add failing test for target not found
- [x] 1.5 Add failing test for API error

## 2. Data Types

- [x] 2.1 Extend `WorkspaceAttributes` in `src/commands/workspaces.rs` to include `settings: serde_json::Map<String, serde_json::Value>` with `#[serde(default)]`
- [x] 2.2 Add `WorkspaceShowOutput` struct with `id`, `slug`, `name`, `settings` fields
- [x] 2.3 Implement `CommandOutput` for `WorkspaceShowOutput` (plain: labeled fields + sorted settings table; JSON: serialize as-is)

## 3. Command Implementation

- [x] 3.1 Add `pub async fn show(ctx: &AppContext, target: &str, out: &Output) -> Result<()>` to `src/commands/workspaces.rs`
- [x] 3.2 Detect UUID vs slug in `show`: parse with `uuid::Uuid::parse_str`, match on `id` for UUID or `slug` otherwise
- [x] 3.3 Return error when no workspace matches target

## 4. CLI Wiring

- [x] 4.1 Add `WorkspacesShowArgs { target: String }` to `src/cli.rs`
- [x] 4.2 Add `Show(WorkspacesShowArgs)` variant to `WorkspacesCommands` enum in `src/cli.rs`
- [x] 4.3 Wire `WorkspacesCommands::Show` in `src/main.rs` to call `workspaces::show`

## 5. Docs & Quality

- [x] 5.1 Update README.md with `workspaces show` usage examples
- [x] 5.2 Run `cargo clippy` and `cargo fmt` and fix any issues
- [x] 5.3 Run `cargo test` and confirm all tests pass
