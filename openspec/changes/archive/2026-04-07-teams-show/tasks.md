## 1. Tests

- [x] 1.1 Add failing test `test_show_by_uuid_plain` in `src/commands/teams.rs`
- [x] 1.2 Add failing test `test_show_by_abbr_plain` in `src/commands/teams.rs`
- [x] 1.3 Add failing test `test_show_json_output` in `src/commands/teams.rs`
- [x] 1.4 Add failing test `test_show_target_not_found` in `src/commands/teams.rs`
- [x] 1.5 Add failing test `test_show_api_error` in `src/commands/teams.rs`

## 2. CLI Wiring

- [x] 2.1 Add `TeamsShowArgs` struct with `target: String` and `json: bool` to `src/cli.rs`
- [x] 2.2 Add `Show(TeamsShowArgs)` variant to `TeamsCommands` enum in `src/cli.rs`
- [x] 2.3 Dispatch `TeamsCommands::Show` to `commands::teams::show` in `src/main.rs`

## 3. Implementation

- [x] 3.1 Add `TeamShowOutput` struct implementing `CommandOutput` (plain: id/abbr/name, json: `{id, abbr, name}`) in `src/commands/teams.rs`
- [x] 3.2 Add `pub async fn show(ctx, target, out)` in `src/commands/teams.rs` that fetches `GET /teams`, matches by `id` then `abbr`, and prints output or error

## 4. Docs & Quality

- [x] 4.1 Update `README.md` to document `rw teams show <target>`
- [x] 4.2 Run `cargo clippy` and fix any warnings
- [x] 4.3 Run `cargo fmt`
- [x] 4.4 Run `cargo test` and confirm all tests pass
