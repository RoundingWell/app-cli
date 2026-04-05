## 1. Dependencies

- [x] 1.1 Add `validator` crate (with `derive` feature) to `Cargo.toml`

## 2. CLI Definition

- [x] 2.1 Add `Update` variant to `CliniciansCommands` enum in `src/cli.rs` with `target`, `field`, and optional `value` args (`--value` is optional to support clearing credentials)
- [x] 2.2 Add match arm for `CliniciansCommands::Update` in `src/main.rs` routing to the new handler

## 3. Target Resolution

- [x] 3.1 Add `resolve_me()` helper in `src/commands/clinicians.rs` that resolves `"me"` to the authenticated clinician's UUID (via `/clinicians/me` or equivalent endpoint)
- [x] 3.2 Update target resolution in the `update` function to dispatch to `resolve_me()`, `resolve_uuid_by_email()`, or pass through UUID directly

## 4. Field Validation

- [x] 4.1 Define a `ClinicianUpdateInput` struct with `#[derive(Validate)]` and per-field annotations:
  - `name`: `#[validate(length(min = 1))]`
  - `email`: `#[validate(email)]`
  - `npi`: `#[validate(length(equal = 10), regex(path = *NPI_RE))]` with a `[0-9]{10}` lazy static
- [x] 4.2 Implement `validate_field(field, value)` that populates the relevant struct field and calls `.validate()`; skip NPI format validation when value is empty/omitted (pass `null` through directly)
- [x] 4.3 Return a clear error for unrecognized field names listing allowed values

## 5. Data Model

- [x] 5.1 Add `npi: Option<String>` and `credentials: Vec<String>` to `ClinicianAttributes` in `src/commands/clinicians.rs`

## 6. API Call

- [x] 6.1 Implement `patch_clinician_attribute()` helper that builds the JSON:API PATCH body for a single field and sends it to `PATCH /clinicians/<uuid>`
- [x] 6.2 For `credentials` field, split the value on commas before building the body (send as `string[]`); empty or omitted value sends `[]`
- [x] 6.3 Parse the full clinician resource from the PATCH response body into `ClinicianOutput` (no follow-up GET needed)

## 7. Output

- [x] 7.1 Implement `update()` public async function in `src/commands/clinicians.rs` wiring together resolution, validation, PATCH, and output
- [x] 7.2 Print the updated clinician using `out.print()` (plain: `"<name> (<id>) updated <field>"`, JSON: full resource)

## 8. Tests

- [x] 8.1 Write test: update by UUID (mock PATCH endpoint, assert body and output)
- [x] 8.2 Write test: update by email (mock GET clinicians list + PATCH, assert resolution and output)
- [x] 8.3 Write test: update with target `"me"` (mock me-resolution endpoint + PATCH)
- [x] 8.4 Write test: validation rejects empty name
- [x] 8.5 Write test: validation rejects invalid email
- [x] 8.6 Write test: validation rejects non-empty npi that is not exactly 10 digits (too short, too long, non-numeric)
- [x] 8.7 Write test: omitted or empty `--value` with `--field npi` sends `null` in PATCH body
- [x] 8.8 Write test: omitted or empty `--value` with `--field credentials` sends `[]` in PATCH body
- [x] 8.9 Write test: credentials split on comma produces correct array in PATCH body
- [x] 8.10 Write test: unsupported field name returns error
- [x] 8.11 Write test: API error response surfaced to caller

## 9. Documentation

- [x] 9.1 Update `README.md` with `clinicians update` usage and examples
- [x] 9.2 Update `docs/` if a clinicians command reference page exists
