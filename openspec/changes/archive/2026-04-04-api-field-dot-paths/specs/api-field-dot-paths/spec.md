## ADDED Requirements

### Requirement: Dot-path field keys produce nested JSON objects
The `--field`/`-f` flag SHALL accept dot-separated keys (e.g. `attributes.name`) and expand them into nested JSON objects in the request body. A flat key with no dots SHALL behave identically to the current behaviour.

#### Scenario: Single dot-path field
- **WHEN** the user passes `-f attributes.name="John Doe"`
- **THEN** the request body SHALL be `{"attributes": {"name": "John Doe"}}`

#### Scenario: Multiple dot-path fields sharing a common prefix
- **WHEN** the user passes `-f attributes.first="John" -f attributes.last="Doe"`
- **THEN** the request body SHALL be `{"attributes": {"first": "John", "last": "Doe"}}`

#### Scenario: Deeply nested dot-path field
- **WHEN** the user passes `-f relationships.team.data.id=<uuid>`
- **THEN** the request body SHALL be `{"relationships": {"team": {"data": {"id": "<uuid>"}}}}`

#### Scenario: Mixed flat and dot-path fields
- **WHEN** the user passes `-f type=clinicians -f attributes.name="Jane"`
- **THEN** the request body SHALL be `{"type": "clinicians", "attributes": {"name": "Jane"}}`

#### Scenario: Dot in value is preserved
- **WHEN** the user passes `-f attributes.email=user@example.com`
- **THEN** the request body SHALL be `{"attributes": {"email": "user@example.com"}}` (the value is not split on dots)

### Requirement: Key path conflict is reported as an error
The system SHALL return a clear error when a dot-path key conflicts with an already-set leaf value (i.e. one field sets `foo` as a leaf and another attempts to descend into `foo.<child>`).

#### Scenario: Conflicting flat and nested keys
- **WHEN** the user passes `-f foo=bar -f foo.baz=qux`
- **THEN** the command SHALL exit with a non-zero status and print an error indicating the key conflict at `foo`
