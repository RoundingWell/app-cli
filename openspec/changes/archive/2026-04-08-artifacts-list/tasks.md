## 1. Tests

- [x] 1.1 Add failing test: `artifacts list` with all filters returns table output
- [x] 1.2 Add failing test: `artifacts list` with empty result displays headers-only table
- [x] 1.3 Add failing test: `artifacts list --json` returns JSON with `data` array
- [x] 1.4 Add failing test: `artifacts list` with API error exits non-zero

## 2. CLI Args

- [x] 2.1 Add `ArtifactsArgs`, `ArtifactsListArgs` structs to `src/cli.rs`
- [x] 2.2 Add `ArtifactsCommands` enum with `List` variant to `src/cli.rs`
- [x] 2.3 Add `Artifacts(ArtifactsArgs)` variant to `Commands` enum in `src/cli.rs`

## 3. Command Implementation

- [x] 3.1 Create `src/commands/artifacts.rs` with JSON:API deserialization structs for artifact resources
- [x] 3.2 Implement `list` function that calls `GET /artifacts` with `filter[type]`, `filter[path]`, `filter[term]` query params
- [x] 3.3 Implement `CommandOutput` for `ArtifactListOutput` displaying `artifact`, `identifier`, `values` as a markdown table
- [x] 3.4 Add `pub mod artifacts;` to `src/commands/mod.rs`

## 4. Wiring

- [x] 4.1 Import `ArtifactsCommands` in `src/main.rs` and add `Commands::Artifacts` dispatch arm

## 5. Docs

- [x] 5.1 Add `artifacts list` to README.md command reference
