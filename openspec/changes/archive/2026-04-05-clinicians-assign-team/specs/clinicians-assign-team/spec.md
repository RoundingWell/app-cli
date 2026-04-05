## ADDED Requirements

### Requirement: Assign a clinician to a team
The CLI SHALL provide a `clinicians assign <target> <team>` subcommand that assigns a clinician to a team, where the team can be identified by UUID, abbreviated name (case-insensitive), or full name (case-insensitive).

#### Scenario: Assign by clinician email and team name
- **WHEN** the user runs `rw clinicians assign <email> <team-name>`
- **THEN** the system finds the clinician by email, finds the team by name (case-insensitive), assigns the team via the API, and confirms success with a plain-text message

#### Scenario: Assign by clinician UUID and team UUID
- **WHEN** the user runs `rw clinicians assign <clinician-uuid> <team-uuid>`
- **THEN** the system uses the UUIDs directly and assigns the team via the API, confirming success with a plain-text message

#### Scenario: Assign by clinician email and team abbr
- **WHEN** the user runs `rw clinicians assign <email> <team-abbr>` and the value matches a team's abbreviated name (case-insensitive)
- **THEN** the system finds the team by abbr, assigns it via the API, and confirms success with a plain-text message

#### Scenario: Team resolved by abbr before name
- **WHEN** the team target is not a UUID and matches both an `abbr` on one team and a `name` on another team
- **THEN** the system resolves to the team matched by `abbr`

#### Scenario: Team not found returns an error
- **WHEN** the user runs `rw clinicians assign <target> <team>` and no team matches the given value by UUID, abbr, or name
- **THEN** the system returns an error indicating the team was not found
