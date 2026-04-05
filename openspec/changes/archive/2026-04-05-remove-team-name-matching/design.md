## Context

`resolve_team` in `src/commands/clinicians.rs` currently resolves a team target in three ways:
1. UUID — direct lookup by `id`
2. Abbr — case-insensitive match on `attributes.abbr`
3. Name — case-insensitive match on `attributes.name` (fallback)

The name fallback adds surface area with no benefit: abbrs are the intended short-form identifiers, and names can be long, unstable, or ambiguous. Tests and docs that exercise name-based resolution are misleading about the CLI's intended contract.

## Goals / Non-Goals

**Goals:**
- Remove name-based fallback from `resolve_team`; accept only UUID or abbr
- Update the error message to reflect the reduced resolution modes
- Remove or replace all test cases that test name-based resolution
- Update docs and spec to remove name-matching language

**Non-Goals:**
- Changing how clinicians or other resources are resolved
- Adding new resolution modes (e.g., email-based team lookup)
- Changing the teams API or response shape

## Decisions

**Remove name matching entirely rather than deprecating it.**
A deprecation path would require a flag or warning mechanism. Since this is a CLI tool without a stable public API contract, a clean removal is simpler and more correct. The breaking change is called out in the proposal.

**Keep abbr matching case-insensitive.**
Abbrs like `NUR`, `PHS`, `OT` are conventionally uppercase but users may type lowercase. Case-insensitivity is already implemented and should be preserved.

**Update the error message** from `"no team found with abbr or name '...'"` to `"no team found with uuid or abbr '...'"` to accurately describe the resolution modes.

## Risks / Trade-offs

**Existing users who pass a team full name will get an error** → This is intentional and called out as BREAKING. The fix is to switch to the team abbr.

**Tests that mock name-based resolution will need replacement** → Any test using a team `name` as a resolution target (e.g., `"nurse"`, `"other"`) must be updated to use an abbr (`NUR`, `PHS`, `OT`). Test helpers like `team_list_response` that conflate `name` and `abbr` fields should be corrected.
