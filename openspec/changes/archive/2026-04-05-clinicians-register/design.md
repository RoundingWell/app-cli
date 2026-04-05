## Context

The CLI manages clinicians via a REST API using JSON:API format. Existing subcommands (`update`, `assign`, `grant`) follow a consistent pattern: parse CLI args in `src/cli.rs`, dispatch in `src/commands/mod.rs`, and implement logic in `src/commands/clinicians.rs`.

The new `clinicians register` command creates a new clinician via `POST /clinicians`. It mirrors the `clinicians update` command in structure but issues a creation request instead of a mutation.

## Goals / Non-Goals

**Goals:**
- Add `clinicians register <email> <name>` subcommand with `--role` and `--team` options
- Accept `--role` as a role UUID or name (resolved via role list API, same as `grant`)
- Accept `--team` as a team UUID, abbreviated name, or full name (resolved via team list API, same as `assign`)
- Issue a `POST /clinicians` request with a JSON:API body
- Print the newly created clinician on success

**Non-Goals:**
- Setting credentials or NPI at registration time (can be done via `clinicians update` afterwards)
- Dry-run mode
- Interactive/prompted input

## Decisions

### Follow existing command structure
Add `Register(CliniciansRegisterArgs)` to `CliniciansCommands` in `src/cli.rs` and a `register` function in `src/commands/clinicians.rs`. This keeps the pattern consistent with all other subcommands.

**Alternative**: A separate file for register logic. Rejected — the existing file is already well-organized and all clinician logic lives together.

### Role and team are optional flags, not positional args
`--role` and `--team` are optional — a clinician can be created without them and assigned later via `grant`/`assign`. This matches the API, which does not require role or team at creation time.

### Role and team resolved before POST
If `--role` or `--team` is provided, resolve them to UUIDs first (using existing `resolve_role` and `resolve_team` helpers), then include them in the POST body as JSON:API relationships. This avoids creating a clinician and then failing on a bad role/team reference.

### POST body uses JSON:API format with relationships
The `POST /clinicians` body includes `attributes` (email, name) and optionally `relationships` (role, team). This follows the JSON:API spec and is consistent with how the PATCH body is structured for updates.

## Risks / Trade-offs

- [Partial creation] If the POST succeeds but role/team resolution had already passed, there is no rollback. However, since resolution happens before the POST, this risk is minimal.
- [Email already exists] The API will return a 4xx error; the CLI surfaces this to the user via the standard error handler.
