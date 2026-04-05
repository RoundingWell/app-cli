## MODIFIED Requirements

### Requirement: Assign a clinician to a team
The CLI SHALL provide a `clinicians assign <target> <team>` subcommand that assigns a clinician to a team, where the team can be identified by UUID or abbreviated name (case-insensitive).

#### Scenario: Assign by clinician UUID and team UUID
- **WHEN** the user runs `rw clinicians assign <clinician-uuid> <team-uuid>`
- **THEN** the system uses the UUIDs directly and assigns the team via the API, confirming success with a plain-text message

#### Scenario: Assign by clinician email and team abbr
- **WHEN** the user runs `rw clinicians assign <email> <team-abbr>` and the value matches a team's abbreviated name (case-insensitive)
- **THEN** the system finds the team by abbr, assigns it via the API, and confirms success with a plain-text message

#### Scenario: Team not found returns an error
- **WHEN** the user runs `rw clinicians assign <target> <team>` and no team matches the given value by UUID or abbr
- **THEN** the system returns an error indicating the team was not found

## REMOVED Requirements

### Requirement: Assign by clinician email and team name
**Reason**: Full name matching is ambiguous and fragile. Team abbrs (e.g., `NUR`, `PHS`, `OT`) are the canonical short-form identifiers.
**Migration**: Use the team's abbreviated name (abbr) instead of the full name. For example, use `NUR` instead of `Nursing`.

### Requirement: Team resolved by abbr before name
**Reason**: Name-based resolution is being removed entirely, making abbr-vs-name priority ordering obsolete.
**Migration**: No action needed; all resolution is now by UUID or abbr only.
