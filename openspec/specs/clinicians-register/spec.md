### Requirement: Register a new clinician
The `clinicians register` subcommand SHALL create a new clinician resource via a `POST /clinicians` request. The command SHALL accept positional arguments `<email>` and `<name>`, an optional `--role` flag (UUID or name), and an optional `--team` flag (UUID, abbreviation, or full name).

#### Scenario: Successful registration without role or team
- **WHEN** the user runs `rw clinicians register jane@example.com "Jane Doe"`
- **THEN** the CLI SHALL POST to `/clinicians` with the given email and name and print the newly created clinician's id, name, and email

#### Scenario: Successful registration with role
- **WHEN** the user runs `rw clinicians register jane@example.com "Jane Doe" --role "Staff"`
- **THEN** the CLI SHALL resolve the role name to a UUID, include it as a relationship in the POST body, and print the created clinician

#### Scenario: Successful registration with team
- **WHEN** the user runs `rw clinicians register jane@example.com "Jane Doe" --team "ICU"`
- **THEN** the CLI SHALL resolve the team name/abbreviation to a UUID, include it as a relationship in the POST body, and print the created clinician

#### Scenario: Successful registration with role and team
- **WHEN** the user runs `rw clinicians register jane@example.com "Jane Doe" --role "Staff" --team "ICU"`
- **THEN** the CLI SHALL resolve both, include both relationships in the POST body, and print the created clinician

#### Scenario: Invalid role rejected before POST
- **WHEN** `--role` is provided and cannot be resolved to a known role
- **THEN** the CLI SHALL exit with a non-zero status and print an error without making a POST request

#### Scenario: Invalid team rejected before POST
- **WHEN** `--team` is provided and cannot be resolved to a known team
- **THEN** the CLI SHALL exit with a non-zero status and print an error without making a POST request

#### Scenario: API error surfaced to user
- **WHEN** the API returns a non-2xx response to the POST request
- **THEN** the CLI SHALL exit with a non-zero status and print the HTTP status code and response body

### Requirement: Input validation before API call
The `<email>` and `<name>` arguments SHALL be validated client-side before any API call is made. Invalid values SHALL produce a clear error without making a network call.

#### Scenario: name must be non-empty
- **WHEN** `<name>` is empty or whitespace-only
- **THEN** the CLI SHALL exit with a non-zero status and report that name must be non-empty

#### Scenario: email must have valid format
- **WHEN** `<email>` does not contain `@` and a domain
- **THEN** the CLI SHALL exit with a non-zero status and report that the value is not a valid email address

### Requirement: POST body in JSON:API format
The POST request SHALL conform to JSON:API format with `type` set to `"clinicians"` and `attributes` containing `email` and `name`. When `--role` or `--team` is provided, the corresponding relationship SHALL be included under `relationships`.

#### Scenario: POST body structure without relationships
- **WHEN** neither `--role` nor `--team` is provided
- **THEN** the request body SHALL be `{"data": {"type": "clinicians", "attributes": {"email": "<email>", "name": "<name>"}}}`

#### Scenario: POST body structure with role relationship
- **WHEN** `--role` is provided and resolved to `<role-uuid>`
- **THEN** the request body SHALL include `"relationships": {"role": {"data": {"type": "roles", "id": "<role-uuid>"}}}`

#### Scenario: POST body structure with team relationship
- **WHEN** `--team` is provided and resolved to `<team-uuid>`
- **THEN** the request body SHALL include `"relationships": {"team": {"data": {"type": "teams", "id": "<team-uuid>"}}}`

#### Scenario: POST response parsed for output
- **WHEN** the API returns a 2xx response
- **THEN** the CLI SHALL parse the clinician resource from the response body and print it without issuing an additional GET request

### Requirement: Prepare a clinician for onboarding
The `clinicians prepare` subcommand SHALL assign a role, team, and workspace access to an existing clinician. For staff clinicians (identified by a `@roundingwell.com` email domain), the command SHALL always use `role="rw"` and `team="OT"`. For non-staff clinicians, the command SHALL use the active profile's `default.role` and `default.team` config values, falling back to `role="employee"` and `team="NUR"` when those values are not set.

#### Scenario: Prepare non-staff clinician with no config default set
- **WHEN** neither `default.role` nor `default.team` is set in the active profile and the clinician email does not end with `@roundingwell.com`
- **THEN** the CLI SHALL use `role="employee"` and `team="NUR"` and proceed with the prepare workflow

#### Scenario: Prepare non-staff clinician with config default set
- **WHEN** `default.role="physician"` and `default.team="ICU"` are set in the active profile and the clinician email does not end with `@roundingwell.com`
- **THEN** the CLI SHALL use `role="physician"` and `team="ICU"` instead of the hard-coded fallbacks and proceed with the prepare workflow

#### Scenario: Prepare staff clinician ignores non-staff default
- **WHEN** the clinician email ends with `@roundingwell.com`
- **THEN** the CLI SHALL use `role="rw"` and `team="OT"` regardless of any config default and proceed with the prepare workflow
