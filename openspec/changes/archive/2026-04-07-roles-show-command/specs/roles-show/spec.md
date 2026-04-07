## ADDED Requirements

### Requirement: Show a role by UUID or name
The CLI SHALL provide a `roles show <target>` subcommand that accepts either a role UUID or role name, resolves the matching role from `GET /roles`, and displays the role's id, name, label, description, and permissions.

#### Scenario: Show role by UUID (plain output)
- **WHEN** the user runs `rw roles show <uuid>`
- **THEN** the system calls `GET /roles`, finds the role with the matching id, and displays id, name, label, description, and a list of permissions sorted alphabetically in plain text

#### Scenario: Show role by name (plain output)
- **WHEN** the user runs `rw roles show <name>` with a valid role name
- **THEN** the system calls `GET /roles`, finds the role with the matching name, and displays the role details in plain text with permissions sorted alphabetically

#### Scenario: Show role as JSON
- **WHEN** the user runs `rw roles show <target> --json`
- **THEN** the system outputs a JSON object with `id`, `name`, `label`, `description`, and `permissions` (array of strings sorted alphabetically)

#### Scenario: Target not found
- **WHEN** the user runs `rw roles show <target>` and no role matches
- **THEN** the system exits with an error indicating no role was found for the given target

#### Scenario: API error on list
- **WHEN** the API returns a non-2xx response for `GET /roles`
- **THEN** the system exits with an error indicating the API status code and body

#### Scenario: Unauthenticated user
- **WHEN** the user runs `rw roles show <target>` without being logged in
- **THEN** the system exits with an error prompting the user to run `rw auth login`

### Requirement: resolve_role is defined in the roles module
The `resolve_role` helper function SHALL be defined in the roles module and used by the clinicians module via import.

#### Scenario: Clinicians commands resolve roles via roles module
- **WHEN** a clinicians command (grant, assign, register, prepare) resolves a role target
- **THEN** it uses the `resolve_role` function imported from the roles module, with no behavior change
