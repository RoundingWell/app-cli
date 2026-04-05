## Context

The `clinicians` command group already has `grant` for assigning roles and `prepare` for full clinician setup (which includes team assignment internally). However, there is no standalone command to assign a clinician to a team after initial setup.

The existing `resolve_team` helper in `src/commands/clinicians.rs` only matches teams by name. The `TeamAttributes` struct only carries `name`; it lacks an `abbr` field. The `patch_clinician_prepare` API call already sends a team relationship, confirming the API supports team assignment via JSON:API relationships.

## Goals / Non-Goals

**Goals:**
- Add `clinicians assign <target> <team>` subcommand
- Resolve `<team>` by UUID (exact), abbr (case-insensitive), or name (case-insensitive), in that priority order
- Reuse existing `resolve_clinician` and API client patterns

**Non-Goals:**
- Assigning multiple teams at once
- Removing or replacing an existing team assignment
- Changing how `prepare` resolves teams

## Decisions

### Extend `TeamAttributes` with `abbr`
Add an `abbr: String` field to `TeamAttributes` so it can be used for resolution. `abbr` is a required field — it is always present and never blank.  
**Alternative**: Keep the struct minimal and do a separate API call. Rejected — the teams list endpoint already returns `abbr`; deserializing it is free.

### Extend `resolve_team` to handle UUID and abbr
Update the existing `resolve_team` function signature to accept a general `target` string and check in order: (1) UUID exact match on `id`, (2) `abbr` case-insensitive match, (3) `name` case-insensitive match.  
**Alternative**: Create a separate `resolve_team_for_assign` function. Rejected — the existing function is private and called in one place (`prepare`); updating it in-place keeps things DRY and doesn't break callers since the behavior is a strict superset.

### New `patch_clinician_team` API call
Add a dedicated `PATCH /clinicians/:id` function that sends only the `team` relationship, mirroring `patch_clinician_role` which sends only the `role` relationship.

### Output struct `GrantTeamOutput`
Follow the same output pattern as `GrantRoleOutput`: a plain-text success message with clinician name/id and team name.

## Risks / Trade-offs

- `abbr` is a required field on all teams, so `TeamAttributes.abbr` can be a plain `String` with no `Option` or default needed.

