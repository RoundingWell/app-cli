## 1. Model Updates

- [x] 1.1 Add `abbr: String` field to `TeamAttributes` struct in `src/commands/clinicians.rs`

## 2. Team Resolution

- [x] 2.1 Update `resolve_team` to accept a general `target` string and resolve by: UUID (id match) → abbr (case-insensitive) → name (case-insensitive)

## 3. API Call

- [x] 3.1 Add `patch_clinician_team` async function that sends a `PATCH /clinicians/:id` with only the `team` relationship (mirrors `patch_clinician_role`)

## 4. Output

- [x] 4.1 Add `AssignTeamOutput` struct with `Display` impl that prints a plain-text success message with clinician name/id and team name

## 5. Subcommand Wiring

- [x] 5.1 Add `Assign` variant to the `CliniciansSubcommand` enum in `src/cli.rs` with `target` and `team` string args
- [x] 5.2 Add `assign` public async function in `src/commands/clinicians.rs` that orchestrates: resolve clinician → resolve team → patch → print output
- [x] 5.3 Wire `CliniciansSubcommand::Assign` to `commands::clinicians::assign` in `src/main.rs`

## 6. Tests

- [x] 6.1 Write failing unit test for `AssignTeamOutput` display format
- [x] 6.2 Write failing integration test: assign by clinician email and team name (mock HTTP)
- [x] 6.3 Write failing integration test: assign by UUID and team UUID
- [x] 6.4 Write failing integration test: assign by clinician email and team abbr
- [x] 6.5 Write failing integration test: team not found returns error
- [x] 6.6 Implement code to make all tests pass

## 7. Docs

- [x] 7.1 Update README.md to document the `clinicians assign` subcommand
