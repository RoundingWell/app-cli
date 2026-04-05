### Requirement: Update a clinician attribute
The `clinicians update` subcommand SHALL update a single attribute of a clinician resource via a PATCH request to the API. The command SHALL accept a `--field` flag naming the attribute and a `--value` flag providing the new value.

#### Scenario: Successful update by UUID
- **WHEN** the user runs `rw clinicians update <uuid> --field name --value "Jane Doe"`
- **THEN** the CLI SHALL PATCH the clinician resource with the new name and print the updated clinician

#### Scenario: Successful update by email
- **WHEN** the user runs `rw clinicians update jane@example.com --field email --value "jane2@example.com"`
- **THEN** the CLI SHALL resolve the email to a UUID, PATCH the resource, and print the updated clinician

#### Scenario: Successful update as authenticated user
- **WHEN** the user runs `rw clinicians update me --field name --value "Jane Doe"`
- **THEN** the CLI SHALL resolve `"me"` to the currently authenticated clinician's UUID, PATCH the resource, and print the updated clinician

#### Scenario: Unsupported field rejected
- **WHEN** the user specifies `--field` with a value not in `{name, email, npi, credentials}`
- **THEN** the CLI SHALL exit with a non-zero status and print an error listing the allowed fields

### Requirement: Target resolution
The `<target>` argument SHALL be resolved to a clinician UUID before making any API call. Three forms are accepted: `"me"` (authenticated user), a valid email address, or a UUID string.

#### Scenario: UUID target passed through
- **WHEN** `<target>` is a valid UUID string
- **THEN** the CLI SHALL use it directly as the clinician ID without any lookup

#### Scenario: Email target resolved
- **WHEN** `<target>` is an email address (contains `@`)
- **THEN** the CLI SHALL fetch the clinician list and resolve the matching clinician's UUID

#### Scenario: "me" target resolved
- **WHEN** `<target>` is the literal string `"me"`
- **THEN** the CLI SHALL resolve to the UUID of the currently authenticated user

#### Scenario: Unknown target
- **WHEN** `<target>` does not match a UUID, email, or `"me"`, and no clinician is found
- **THEN** the CLI SHALL exit with a non-zero status and print a descriptive error

### Requirement: Field validation before API call
Each field SHALL be validated client-side before sending the PATCH request. Invalid values SHALL produce a clear error without making a network call.

#### Scenario: name must be non-empty
- **WHEN** `--field name` is given with an empty or whitespace-only `--value`
- **THEN** the CLI SHALL exit with a non-zero status and report that name must be non-empty

#### Scenario: email must have valid format
- **WHEN** `--field email` is given and `--value` does not contain `@` and a domain
- **THEN** the CLI SHALL exit with a non-zero status and report that the value is not a valid email address

#### Scenario: npi must be exactly 10 digits when provided
- **WHEN** `--field npi` is given with a non-empty `--value` that is not exactly 10 decimal digits
- **THEN** the CLI SHALL exit with a non-zero status and report that NPI must be a 10-digit number

#### Scenario: valid npi accepted
- **WHEN** `--field npi` is given with a string of exactly 10 decimal digits (e.g. `"1234567890"`)
- **THEN** the CLI SHALL send the value to the API

#### Scenario: npi cleared when value is empty or omitted
- **WHEN** `--field npi` is given with an empty `--value` or with `--value` omitted
- **THEN** the CLI SHALL send `null` as the npi value in the PATCH body

#### Scenario: credentials cleared when value is empty or omitted
- **WHEN** `--field credentials` is given with an empty `--value` or with `--value` omitted
- **THEN** the CLI SHALL send `[]` as the credentials array in the PATCH body

#### Scenario: credentials split on comma
- **WHEN** `--field credentials --value "RN,MD"` is provided
- **THEN** the CLI SHALL send `["RN", "MD"]` as the credentials array in the PATCH body

### Requirement: API PATCH in JSON:API format
The PATCH request SHALL conform to JSON:API format, including only the field being updated under `attributes`.

#### Scenario: PATCH body structure
- **WHEN** the update command sends a request
- **THEN** the request body SHALL be `{"data": {"type": "clinicians", "id": "<uuid>", "attributes": {"<field>": <value>}}}`

#### Scenario: PATCH response contains updated clinician
- **WHEN** the API returns a 2xx response to the PATCH request
- **THEN** the CLI SHALL parse the full clinician resource from the response body (no additional GET required)

#### Scenario: API error surfaced to user
- **WHEN** the API returns a non-2xx response
- **THEN** the CLI SHALL exit with a non-zero status and print the HTTP status code and response body
