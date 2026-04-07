## Requirements

### Requirement: Workspaces can be listed
The CLI SHALL provide a `workspaces list` subcommand that fetches all workspaces from the API and displays them in a table sorted by `name`.

#### Scenario: Successful list
- **WHEN** the user runs `rw workspaces list`
- **THEN** the CLI fetches `GET /workspaces` using the active profile credentials
- **THEN** the CLI displays a table with columns `id`, `slug`, `name` sorted alphabetically by `name`

#### Scenario: Empty list
- **WHEN** the user runs `rw workspaces list` and the API returns no workspaces
- **THEN** the CLI displays an empty table (headers only)

#### Scenario: JSON output
- **WHEN** the user runs `rw workspaces list --json`
- **THEN** the CLI outputs a JSON object with a `data` array where each element has `id`, `slug`, and `name` fields

#### Scenario: API error
- **WHEN** the API returns a non-2xx response
- **THEN** the CLI exits with a non-zero status and prints an error message
