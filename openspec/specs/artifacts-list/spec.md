## ADDED Requirements

### Requirement: Artifacts can be listed by type, path, and term
The CLI SHALL provide an `artifacts list <type>` subcommand that accepts required `--path` and `--term` options, fetches matching artifacts from the API, and displays them grouped by identifier. The `values` attribute is a `map<string, mixed>` (string keys, mixed values).

#### Scenario: Successful list
- **WHEN** the user runs `rw artifacts list <type> --path=<path> --term=<term>`
- **THEN** the CLI sends `GET /artifacts?filter[type]=<type>&filter[path]=<path>&filter[term]=<term>` using the active profile credentials
- **THEN** the CLI displays each artifact as its `identifier` followed by a markdown table of its `values` (key/value rows), with a blank line between artifacts

#### Scenario: Empty list
- **WHEN** the user runs `rw artifacts list <type> --path=<path> --term=<term>` and the API returns no artifacts
- **THEN** the CLI displays no output

#### Scenario: JSON output
- **WHEN** the user runs `rw artifacts list <type> --path=<path> --term=<term> --json`
- **THEN** the CLI outputs a JSON object with a `data` array where each element has `artifact`, `identifier`, and `values` fields, with `values` as a JSON object

#### Scenario: Missing required option
- **WHEN** the user runs `rw artifacts list <type>` without `--path` or `--term`
- **THEN** the CLI exits with a non-zero status and prints a usage error indicating the missing option

#### Scenario: API error
- **WHEN** the API returns a non-2xx response
- **THEN** the CLI exits with a non-zero status and prints an error message
