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

### Requirement: Integer path segments produce JSON arrays
When a dot-path key contains a bare integer segment, the system SHALL produce a JSON array at that position rather than an object with an integer string key. This is the behavior provided by `json_dotpath::dot_set`.

#### Scenario: Single integer segment creates array element
- **WHEN** the user passes `-f items.0.id=abc`
- **THEN** the request body SHALL be `{"items": [{"id": "abc"}]}`

#### Scenario: Multiple integer segments at the same path merge into one array
- **WHEN** the user passes `-f items.0.id=abc -f items.1.id=def`
- **THEN** the request body SHALL be `{"items": [{"id": "abc"}, {"id": "def"}]}`

#### Scenario: Integer segment nested under dot-path object key
- **WHEN** the user passes `-f relationships.workspaces.data.0.type=workspaces -f relationships.workspaces.data.0.id=uuid-123`
- **THEN** the request body SHALL be `{"relationships": {"workspaces": {"data": [{"type": "workspaces", "id": "uuid-123"}]}}}`

### Requirement: Key path conflict is reported as an error
The system SHALL return a non-zero exit status and emit an error when a dot-path key conflicts with an already-set value. The error message SHALL be `"Unable to set field {key} because it conflicts with another field"`, where `{key}` is the full dot-path key that could not be set.

#### Scenario: Conflicting flat and nested keys
- **WHEN** the user passes `-f foo=bar -f foo.baz=qux`
- **THEN** the command SHALL exit with a non-zero status and print the error `"Unable to set field foo.baz because it conflicts with another field"`

#### Scenario: Multi-level conflict reports the full key
- **WHEN** the user passes `-f a.b=x -f a.b.c=y`
- **THEN** the command SHALL exit with a non-zero status and print the error `"Unable to set field a.b.c because it conflicts with another field"`
