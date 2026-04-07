## Context

The CLI currently calls `GET /roles` in `src/commands/clinicians.rs` to resolve role names/IDs for the `clinicians grant` and `clinicians prepare` commands. The `RoleAttributes`, `RoleResource`, and `RoleListResponse` structs live privately in that module and only surface `name`. No public `roles list` command exists.

## Goals / Non-Goals

**Goals:**
- Add a `roles list` command that calls `GET /roles` and displays `id`, `name`, and `label`, sorted by `label`
- Move role types (`RoleAttributes`, `RoleResource`, `RoleListResponse`) to a new `src/commands/roles.rs` module so they can be shared with `clinicians`
- Add `label` to `RoleAttributes` to expose the human-readable label field from the API

**Non-Goals:**
- Adding role CRUD operations (create, update, delete)
- Filtering or searching roles
- Changing the `clinicians grant` / `clinicians prepare` behavior

## Decisions

**New `roles` module, not inline in `clinicians`**
The role types need to be public and reusable. Extracting them to `src/commands/roles.rs` mirrors the existing `teams.rs` pattern and keeps modules cohesive. The alternative (re-exporting from `clinicians`) would create an awkward dependency direction.

**Add `label` to `RoleAttributes`**
The `GET /roles` API likely returns a `label` field in attributes (human-readable display name). We add it to `RoleAttributes` so both the list command and the `resolve_role` helper benefit. `clinicians` currently only uses `name` for matching; this is unaffected.

**Sort by `label`**
Consistent with the proposal; `label` is the user-facing display name, making it the natural sort key.

## Risks / Trade-offs

- **API response shape** → If `GET /roles` does not return a `label` field, deserialization will fail. Mitigation: use `#[serde(default)]` on `label` or verify against API docs before shipping.
- **`clinicians` import change** → Any code importing `RoleAttributes`/`RoleResource` privately will need updating. Since these were private types, only `clinicians.rs` is affected.
