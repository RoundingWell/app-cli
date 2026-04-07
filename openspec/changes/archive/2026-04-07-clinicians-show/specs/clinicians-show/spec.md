## ADDED Requirements

### Requirement: Show clinician by UUID
The system SHALL fetch and display a single clinician when given a UUID target, by calling `GET /clinicians/:id`.

#### Scenario: Valid UUID target
- **WHEN** user runs `rw clinicians show <uuid>`
- **THEN** the CLI calls `GET /clinicians/:id` and displays the clinician's id, name, email, enabled status, npi, and credentials

#### Scenario: UUID not found
- **WHEN** user runs `rw clinicians show <uuid>` and the API returns a non-success status
- **THEN** the CLI exits with an error message including the HTTP status

### Requirement: Show clinician by email
The system SHALL resolve a clinician by email using `GET /clinicians?filter[email]={email}` and display the result.

#### Scenario: Valid email target
- **WHEN** user runs `rw clinicians show <email>`
- **THEN** the CLI calls `GET /clinicians?filter[email]={email}` and displays the first matching clinician's details

#### Scenario: Email not found
- **WHEN** user runs `rw clinicians show <email>` and the API returns an empty data array
- **THEN** the CLI exits with an error message stating no clinician was found with that email

### Requirement: Show current clinician with "me"
The system SHALL fetch and display the authenticated clinician's own record when given the target `"me"`, by calling `GET /clinicians/me`.

#### Scenario: Authenticated user uses "me" target
- **WHEN** user runs `rw clinicians show me`
- **THEN** the CLI calls `GET /clinicians/me` and displays the clinician's id, name, email, enabled status, npi, and credentials

#### Scenario: Unauthenticated or unauthorized "me" request
- **WHEN** user runs `rw clinicians show me` and the API returns 401 or 403
- **THEN** the CLI exits with an error message including the HTTP status

### Requirement: Show output fields
The system SHALL include id, name, email, enabled, npi, and credentials in the output for all target types.

#### Scenario: JSON output
- **WHEN** user runs `rw clinicians show <target> --json`
- **THEN** the output is a JSON object containing id, name, email, enabled, npi, and credentials fields

#### Scenario: Plain text output
- **WHEN** user runs `rw clinicians show <target>` without `--json`
- **THEN** the output is a human-readable single line summarizing the clinician record
