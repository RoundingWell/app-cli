## ADDED Requirements

### Requirement: List all roles
The CLI SHALL provide a `roles list` subcommand that fetches all roles from the API and displays them sorted by `label`.

#### Scenario: List roles successfully
- **WHEN** the user runs `rw roles list`
- **THEN** the system calls `GET /roles`, displays a table with columns `id`, `name`, and `label`, sorted ascending by `label`

#### Scenario: List roles as JSON
- **WHEN** the user runs `rw roles list --json`
- **THEN** the system outputs the role list as a JSON array with `id`, `name`, and `label` fields, sorted by `label`

#### Scenario: Empty roles list
- **WHEN** the user runs `rw roles list` and the API returns an empty list
- **THEN** the system displays an empty table with no rows

#### Scenario: API error returns an error message
- **WHEN** the user runs `rw roles list` and the API returns a non-2xx response
- **THEN** the system exits with an error indicating the API status code and body

#### Scenario: Unauthenticated user
- **WHEN** the user runs `rw roles list` without being logged in
- **THEN** the system exits with an error prompting the user to run `rw auth login`
