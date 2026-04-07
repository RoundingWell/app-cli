## 1. Tests

- [x] 1.1 Add failing test for `clinicians show <uuid>` — mock `GET /clinicians/:id` returning a clinician resource, assert output fields
- [x] 1.2 Add failing test for `clinicians show <email>` — mock `GET /clinicians?filter[email]={email}` returning a single-item list, assert output fields
- [x] 1.3 Add failing test for `clinicians show me` — mock `GET /clinicians/me` returning a clinician resource, assert output fields
- [x] 1.4 Add failing test for email not found — mock `GET /clinicians?filter[email]={email}` returning empty data, assert error

## 2. CLI Argument Wiring

- [x] 2.1 Add `Show(CliniciansTargetArgs)` variant to `CliniciansCommands` enum in `src/cli.rs`
- [x] 2.2 Add dispatch for `CliniciansCommands::Show` in `src/commands/mod.rs`

## 3. Core Implementation

- [x] 3.1 Add `ClinicianShowOutput` struct (id, name, email, enabled, npi, credentials) with `CommandOutput` impl in `src/commands/clinicians.rs`
- [x] 3.2 Add private `fetch_clinician_me` function calling `GET /clinicians/me` and returning `ClinicianResource`
- [x] 3.3 Add private `fetch_clinician_by_email_filter` function calling `GET /clinicians?filter[email]={email}` and returning `ClinicianResource`
- [x] 3.4 Add public `show` function dispatching on `"me"`, UUID, or email and printing `ClinicianShowOutput`

## 4. Docs

- [x] 4.1 Update README.md with `clinicians show` command documentation
