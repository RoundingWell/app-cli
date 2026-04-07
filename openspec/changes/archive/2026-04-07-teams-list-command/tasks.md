## 1. CLI Types

- [x] 1.1 Add `Teams(TeamsArgs)` variant to `Commands` enum in `src/cli.rs`
- [x] 1.2 Define `TeamsArgs`, `TeamsCommands`, and `TeamsListArgs` structs in `src/cli.rs`

## 2. Teams Command Module

- [x] 2.1 Create `src/commands/teams.rs` and move `TeamAttributes`, `TeamResource`, `TeamListResponse` from `clinicians.rs` into it with `pub(crate)` visibility
- [x] 2.2 Update `clinicians.rs` to import the moved types from `super::teams`
- [x] 2.3 Add `tabled` to `Cargo.toml` dependencies
- [x] 2.4 Define `TeamRow` struct (deriving `tabled::Tabled`) and `TeamListOutput` with `teams: Vec<TeamRow>`
- [x] 2.5 Implement `CommandOutput` for `TeamListOutput`: `plain()` builds a `tabled` table with `Style::markdown()` sorted by `abbr`
- [x] 2.6 Implement `pub async fn list(ctx: &AppContext, out: &Output) -> Result<()>` that calls `GET /teams` and prints the output
- [x] 2.7 Add `pub mod teams;` to `src/commands/mod.rs`

## 3. Wire Up in Main

- [x] 3.1 Handle `Commands::Teams` in `src/main.rs` dispatching to `TeamsCommands::List`

## 4. Tests

- [x] 4.1 Write a test for `list` with a mocked `GET /teams` response returning multiple teams, asserting sorted table output
- [x] 4.2 Write a test for `list` with an empty teams response
- [x] 4.3 Write a test for `list` with a non-2xx API response, asserting error propagation

## 5. Documentation

- [x] 5.1 Update `README.md` to document the `teams list` command
- [x] 5.2 Run `cargo clippy` and `cargo fmt` and resolve any warnings
- [x] 5.3 Run `cargo test` and confirm all tests pass
