## Context

The CLI already has a `clinicians` command module (`src/commands/clinicians.rs`) with subcommands for `assign`, `enable`, `disable`, and `prepare`. These subcommands follow a consistent pattern: resolve a target clinician (UUID or email), build a JSON:API PATCH body, send it, and print the result.

Target resolution today handles UUID or email strings. The `"me"` shorthand is not yet supported anywhere but the API likely exposes a `/clinicians/me` endpoint or the current user can be inferred from auth context.

The four fields to support — `name`, `email`, `npi`, `credentials` — are all `attributes` on the clinician resource. The API accepts PATCH in JSON:API format. `credentials` is an array of strings in the schema, while the others are scalars.

## Goals / Non-Goals

**Goals:**
- Add `rw clinicians update <target> --field <name> --value <val>`
- Support `target` = `"me"`, email address, or UUID
- Validate each field client-side before sending to the API
- Send a JSON:API PATCH to update a single attribute at a time
- Print the updated clinician on success

**Non-Goals:**
- Updating multiple fields in one command invocation (one `--field`/`--value` pair only)
- Adding to or removing individual items from the `credentials` array (the value replaces the entire array)
- Any changes to how other subcommands resolve targets

## Decisions

### 1. Single field per invocation

**Decision:** Accept exactly one `--field`/`--value` pair per call.

**Rationale:** Keeps the CLI surface minimal and consistent with the existing `--field`/`--value` pattern in other commands (e.g., `fields update`). Users wanting multiple field updates can chain commands. A multi-field flag design would require validating pairs and is significantly more complex.

**Alternative considered:** `--name`, `--email`, etc. as individual flags. Rejected because it couples the CLI interface to the attribute names and requires new flags for any future fields.

### 2. `"me"` resolution via `/clinicians` list filtered by auth identity

**Decision:** Resolve `"me"` by calling `GET /clinicians?filter[me]=true` or by fetching `/auth/me` (or equivalent) to get the current user's UUID, then fetching the clinician by that UUID. If the API provides a `/clinicians/me` shorthand, use it directly.

**Rationale:** The existing `require_auth` + `resolve_uuid_by_email` pattern doesn't cover the authenticated-user case. The cleanest solution is to add a `resolve_me()` helper that calls the appropriate endpoint.

**Alternative considered:** Caching the current user's UUID in auth config. Rejected because it would go stale and adds write-back complexity.

### 3. `credentials` value is comma-separated → array

**Decision:** When `--field credentials` is used, `--value` is treated as a comma-separated list of credential strings (e.g., `"RN,MD"`), which is split into an array before sending.

**Rationale:** The API stores credentials as `string[]`. The CLI value must be a single string argument. Comma-separation is a simple and widely-used convention for array inputs in CLI tools.

**Alternative considered:** Repeated `--value` flags for arrays. Rejected because the current `--field`/`--value` signature uses a single value pair; extending to multiple values would require a flag redesign.

### 4. Validation strategy

**Decision:** Use the [`validator`](https://crates.io/crates/validator) crate for client-side validation. Define a struct per field (or a unified `ClinicianUpdateInput` struct) with `#[validate(...)]` annotations, call `validate()` before sending to the API, and fail fast with a clear error.

| Field        | Validation                              |
|--------------|-----------------------------------------|
| `name`       | `#[validate(length(min = 1))]` after trimming |
| `email`      | `#[validate(email)]`                    |
| `npi`        | Empty or omitted `--value` sends `null` (clears NPI); non-empty value must pass `#[validate(length(equal = 10), regex(path = *NPI_RE))]` where `NPI_RE` matches `[0-9]{10}` |
| `credentials`| Empty or omitted `--value` sends `[]` (clears all); non-empty value is split on commas — no `validator` annotation needed |

**Rationale:** The `validator` crate provides declarative, well-tested validation (including RFC-compliant email checking) and standardised error types, avoiding hand-rolled regex for email. It is the conventional Rust choice for struct-level input validation.

**Alternative considered:** Hand-rolled regex / manual checks. Rejected in favour of `validator` to reduce bespoke validation code and leverage its email format support.

## Risks / Trade-offs

- **`"me"` resolution depends on an undiscovered endpoint** → Mitigation: check the OpenAPI spec for a `/clinicians/me` path or an auth profile endpoint; if absent, fetch the full clinician list and match against the authenticated identity.
- **`credentials` comma-split is lossy if a credential itself contains a comma** → Low risk in practice (credential strings like `"RN"`, `"MD"` never contain commas), but worth noting.
- **Single field per call means multiple roundtrips for bulk updates** → Acceptable trade-off given the scope of this feature.
