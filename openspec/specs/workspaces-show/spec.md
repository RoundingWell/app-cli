## Requirements

### Requirement: Show a workspace by UUID or slug
The CLI SHALL provide a `workspaces show <target>` subcommand that accepts either a workspace UUID or slug, resolves the matching workspace from `GET /workspaces`, and displays the workspace's id, slug, name, and a settings table.

#### Scenario: Show workspace by UUID (plain output)
- **WHEN** the user runs `rw workspaces show <uuid>`
- **THEN** the system calls `GET /workspaces`, finds the workspace with the matching id, and displays id, slug, name, and a markdown table of settings (name, value) sorted alphabetically by name

#### Scenario: Show workspace by slug (plain output)
- **WHEN** the user runs `rw workspaces show <slug>` with a valid workspace slug
- **THEN** the system calls `GET /workspaces`, finds the workspace with the matching slug, and displays the workspace details in plain text with a settings table

#### Scenario: Show workspace as JSON
- **WHEN** the user runs `rw workspaces show <target> --json`
- **THEN** the system outputs a JSON object with `id`, `slug`, `name`, and `settings` (object with all settings key/value pairs)

#### Scenario: Target not found
- **WHEN** the user runs `rw workspaces show <target>` and no workspace matches the UUID or slug
- **THEN** the system exits with a non-zero status and prints an error indicating no workspace was found for the given target

#### Scenario: API error on list
- **WHEN** the API returns a non-2xx response for `GET /workspaces`
- **THEN** the system exits with a non-zero status and prints an error indicating the API status code and body

#### Scenario: Empty settings
- **WHEN** the matched workspace has no settings keys
- **THEN** the system displays the id, slug, and name, and an empty settings table (headers only)
