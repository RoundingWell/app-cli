## ADDED Requirements

### Requirement: Teams can be listed
The CLI SHALL provide a `teams list` subcommand that fetches all teams from the API and displays them in a table sorted by `abbr`.

#### Scenario: Successful list
- **WHEN** the user runs `rw teams list`
- **THEN** the CLI fetches `GET /teams` using the active profile credentials
- **THEN** the CLI displays a table with columns `id`, `abbr`, `name` sorted alphabetically by `abbr`

#### Scenario: Empty list
- **WHEN** the user runs `rw teams list` and the API returns no teams
- **THEN** the CLI displays an empty table (headers only)

#### Scenario: JSON output
- **WHEN** the user runs `rw teams list --json`
- **THEN** the CLI outputs a JSON object with a `data` array where each element has `id`, `abbr`, and `name` fields

#### Scenario: API error
- **WHEN** the API returns a non-2xx response
- **THEN** the CLI exits with a non-zero status and prints an error message
