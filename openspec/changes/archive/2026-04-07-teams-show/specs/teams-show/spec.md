## ADDED Requirements

### Requirement: Show a team by UUID or abbreviation
The CLI SHALL provide a `teams show <target>` subcommand that accepts either a team UUID or abbreviation (`abbr`), resolves the matching team from `GET /teams`, and displays the team's `id`, `abbr`, and `name`.

#### Scenario: Show team by UUID (plain output)
- **WHEN** the user runs `rw teams show <uuid>`
- **THEN** the system calls `GET /teams`, finds the team with the matching `id`, and displays `id`, `abbr`, and `name` in plain text

#### Scenario: Show team by abbreviation (plain output)
- **WHEN** the user runs `rw teams show <abbr>` with a valid team abbreviation
- **THEN** the system calls `GET /teams`, finds the team with the matching `abbr`, and displays `id`, `abbr`, and `name` in plain text

#### Scenario: Show team as JSON
- **WHEN** the user runs `rw teams show <target> --json`
- **THEN** the system outputs a JSON object with `id`, `abbr`, and `name`

#### Scenario: Target not found
- **WHEN** the user runs `rw teams show <target>` and no team matches the UUID or abbreviation
- **THEN** the system exits with a non-zero status and prints an error indicating no team was found for the given target

#### Scenario: API error on list
- **WHEN** the API returns a non-2xx response for `GET /teams`
- **THEN** the system exits with a non-zero status and prints an error indicating the API status code and body

#### Scenario: Unauthenticated user
- **WHEN** the user runs `rw teams show <target>` without being logged in
- **THEN** the system exits with an error prompting the user to run `rw auth login`
