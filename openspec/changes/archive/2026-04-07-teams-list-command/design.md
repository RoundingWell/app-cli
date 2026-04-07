## Context

The CLI uses a module-per-resource pattern: `src/commands/clinicians.rs` handles all clinician subcommands, while `src/cli.rs` declares the CLI argument types and `src/commands/mod.rs` registers modules. There is no existing `teams` command module. The `GET /teams` endpoint is already called internally by `clinicians.rs` to resolve team references. The output system uses `CommandOutput` trait with `plain()` for human output and `serde::Serialize` for `--json` output.

## Goals / Non-Goals

**Goals:**
- Add a `teams list` subcommand that fetches all teams and renders a sorted table of `id`, `abbr`, `name`
- Follow existing patterns: new module `src/commands/teams.rs`, new CLI types in `src/cli.rs`, wired through `src/main.rs`
- Table sorted alphabetically by `abbr`

**Non-Goals:**
- Filtering or pagination
- Other teams subcommands (e.g., `teams show`)

## Decisions

### New `teams` command module

Add `src/commands/teams.rs` mirroring the structure of `clinicians.rs`: JSON:API deserialization types, an output type implementing `CommandOutput`, and a public `list` async function.

**Rationale**: Keeps the pattern consistent and avoids bloating `clinicians.rs` with unrelated concerns.

### Table rendering via `tabled` crate with Markdown style

Use the `tabled` crate to render the teams table with `Style::markdown()`. This produces a standard Markdown table that is both human-readable and paste-friendly. The `--json` flag is respected via the existing `Output::print` / `CommandOutput` pattern — `plain()` returns the `tabled` Markdown string; the `Serialize` impl provides the JSON representation.

**Alternative considered**: Manual column-width formatting with `format!`. Rejected in favor of `tabled` for correctness, maintainability, and consistent alignment without custom padding logic.

### Move existing JSON:API types from `clinicians.rs` to `teams.rs`

`TeamAttributes`, `TeamResource`, and `TeamListResponse` are currently private in `clinicians.rs`. Move them to `teams.rs` and re-export or `pub(crate)` them as needed so `clinicians.rs` can import from `teams`. This eliminates duplication and establishes `teams.rs` as the single source of truth for team data types.

**Alternative considered**: Defining equivalent types locally in `teams.rs` and leaving `clinicians.rs` unchanged. Rejected because it creates two parallel definitions of the same API shape.

## Risks / Trade-offs

- [API changes to `/teams` response shape] → Minimal risk; the endpoint is already in use for other commands.
- [Column alignment breaks on very long values] → Mitigated by computing widths dynamically from actual data.
