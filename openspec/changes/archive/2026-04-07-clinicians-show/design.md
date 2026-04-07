## Context

The `clinicians` command has subcommands for mutating clinician records (enable, disable, update, prepare, etc.) but no read-only lookup command. Several internal helpers already exist — `fetch_clinician_by_uuid`, `fetch_clinician_by_email`, and `resolve_me` — but `resolve_me` only returns the UUID, not the full resource. The `fetch_clinician_by_email` helper also does a full list fetch and searches client-side rather than using the API filter.

## Goals / Non-Goals

**Goals:**
- Add `rw clinicians show <target>` that accepts UUID, email, or `"me"`
- For `"me"`: call `GET /clinicians/me` and display the result directly
- For UUID: call `GET /clinicians/:id`
- For email: use `GET /clinicians?filter[email]={email}` (server-side filter per spec)
- Display full clinician details: id, name, email, enabled, npi, credentials

**Non-Goals:**
- Modifying or replacing existing helper functions used by other commands
- Pagination or listing multiple results

## Decisions

### Target dispatch: three-way branch
The `update` command already establishes the pattern: `if target == "me"` → resolve me, `else if Uuid::parse_str(target).is_ok()` → fetch by UUID, `else` → resolve by email. The `show` command follows the same dispatch.

**Alternative considered**: a single `resolve_clinician` helper that returns `ClinicianResource` for all three cases. Rejected for now — the `"me"` endpoint returns the full resource directly, making a unified helper slightly more complex without additional callers benefiting.

### Email resolution: use API filter endpoint
The user requirement specifies `GET /clinicians?filter[email]={email}`. The existing `fetch_clinician_by_email` helper fetches the full list and filters client-side. A new private helper `fetch_clinician_by_email_filter` will use the query parameter approach instead. The existing helper is left intact for use by other commands.

**Alternative considered**: updating the existing `fetch_clinician_by_email` to use the filter. Rejected — it would change behavior for callers (prepare, grant, assign) that may not expect it, and those callers don't need the filter approach.

### `"me"` path: fetch full resource directly
`resolve_me` only returns the UUID; for `show` we need the full `ClinicianResource`. Rather than chaining `resolve_me` → `fetch_clinician_by_uuid`, the `show` command calls `GET /clinicians/me` directly and deserializes the full response. This is one fewer API round-trip.

### Output type: new `ClinicianShowOutput`
A dedicated `ClinicianShowOutput` struct with id, name, email, enabled, npi, and credentials gives clean JSON output and a purpose-built plain text representation.

## Risks / Trade-offs

- `GET /clinicians?filter[email]={email}` may return an empty list if the API does not support that filter parameter → handled by erroring with a clear message
- `GET /clinicians/me` returns HTTP 401/403 when the session token lacks clinician identity → error message propagated from existing auth handling
