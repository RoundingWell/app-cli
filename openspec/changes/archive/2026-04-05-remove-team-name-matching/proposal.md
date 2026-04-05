## Why

Team resolution by full name is ambiguous and fragile — names can change, contain special characters, or collide across contexts. Abbreviations (e.g., `NUR`, `PHS`, `OT`) are stable, short, and purpose-built for CLI use, making name-based matching unnecessary.

## What Changes

- Remove full name (case-insensitive) matching from `resolve_team`
- Teams are resolved only by UUID or `abbr`
- **BREAKING**: `rw clinicians assign <target> <team-name>` no longer resolves by full name; only UUID or abbr values are accepted
- Update tests and docs to reference team abbr values (e.g., `NUR`, `PHS`, `OT`) instead of full names

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `clinicians-assign-team`: Remove full name (case-insensitive) matching from team resolution — teams are identified only by UUID or abbr

## Impact

- `src/` — `resolve_team` function and any callers that rely on name-based fallback
- Tests — remove/replace name-based team resolution test cases; use abbr values (`NUR`, `PHS`, `OT`)
- Docs — update examples that show team resolution by full name
- `openspec/specs/clinicians-assign-team/spec.md` — remove name-matching scenarios and requirement language
