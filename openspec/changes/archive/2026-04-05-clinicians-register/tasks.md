## 1. CLI Arguments

- [x] 1.1 Add `CliniciansRegisterArgs` struct to `src/cli.rs` with `email: String`, `name: String`, `--role: Option<String>`, `--team: Option<String>`
- [x] 1.2 Add `Register(CliniciansRegisterArgs)` variant to `CliniciansCommands` enum in `src/cli.rs`

## 2. Core Implementation

- [x] 2.1 Add `register` async function to `src/commands/clinicians.rs` that validates `email` (valid format) and `name` (non-empty) before any API call, returning an error on failure
- [x] 2.2 Build a JSON:API POST body from the validated email and name
- [x] 2.3 Resolve `--role` to a UUID (using existing `resolve_role`) and include as a relationship in the POST body when provided
- [x] 2.4 Resolve `--team` to a UUID (using existing `resolve_team`) and include as a relationship in the POST body when provided
- [x] 2.5 Issue `POST /clinicians` with the assembled body and parse the response into a `ClinicianRegisterOutput`

## 3. Output

- [x] 3.1 Define `ClinicianRegisterOutput` struct with `id`, `name`, `email` fields and implement `CommandOutput` with a human-readable plain message

## 4. Dispatch

- [x] 4.1 Add `CliniciansCommands::Register` arm to the dispatch match in `src/commands/mod.rs`

## 5. Tests

- [x] 5.1 Write a test for successful registration without role or team (mock POST, assert body and output)
- [x] 5.2 Write a test for registration with `--role` (mock role list + POST, assert relationship in body)
- [x] 5.3 Write a test for registration with `--team` (mock team list + POST, assert relationship in body)
- [x] 5.4 Write a test for blank name (assert non-zero exit, no network call)
- [x] 5.5 Write a test for invalid email format (assert non-zero exit, no network call)
- [x] 5.6 Write a test for invalid role (mock role list returning no match, assert non-zero exit and no POST)
- [x] 5.7 Write a test for API error response (mock POST returning 4xx, assert non-zero exit)

## 6. Docs and Quality

- [x] 6.1 Update `README.md` to document the `clinicians register` subcommand with usage and options
- [x] 6.2 Run `cargo clippy` and fix any warnings
- [x] 6.3 Run `cargo fmt`
- [x] 6.4 Run `cargo test` and confirm all tests pass
