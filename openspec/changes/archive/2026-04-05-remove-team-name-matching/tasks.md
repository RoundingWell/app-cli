## 1. Update `resolve_team` Implementation

- [x] 1.1 Remove the name-based fallback branch from `resolve_team` in `src/commands/clinicians.rs`
- [x] 1.2 Update the error message from `"no team found with abbr or name '...'"` to `"no team found with uuid or abbr '...'"` in the abbr-not-found path

## 2. Update Tests

- [x] 2.1 Remove `test_assign_by_email_and_team_name` test (name-based resolution no longer supported)
- [x] 2.2 Remove `test_assign_team_abbr_takes_priority_over_name` test (abbr-vs-name priority is moot)
- [x] 2.3 Update any test that passes a team full name as a target to use an abbr (`NUR`, `PHS`, or `OT`) instead
- [x] 2.4 Fix `team_list_response` helper (and any similar helpers) to use distinct `name` and `abbr` values so tests accurately reflect the data model
- [x] 2.5 Update the "team not found" error message assertion to match the new wording
- [x] 2.6 Run `cargo test` and confirm all tests pass

## 3. Update Docs

- [x] 3.1 Update README.md to remove any examples or language referencing team resolution by full name
- [x] 3.2 Update any other docs files (`docs/`) that mention name-based team resolution

## 4. Lint and Format

- [x] 4.1 Run `cargo clippy` and fix any warnings
- [x] 4.2 Run `cargo fmt` to ensure consistent formatting
