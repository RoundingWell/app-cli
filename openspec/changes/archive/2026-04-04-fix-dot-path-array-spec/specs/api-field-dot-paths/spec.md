## ADDED Requirements

### Requirement: Integer path segments produce JSON arrays
When a dot-path key contains a bare integer segment, the system SHALL produce a JSON array at that position rather than an object with an integer string key. This is the behavior provided by `json_dotpath::dot_set`.

#### Scenario: Single integer segment creates array element
- **WHEN** the user passes `-f items.0.id=abc`
- **THEN** the request body SHALL be `{"items": [{"id": "abc"}]}`

#### Scenario: Multiple integer segments at the same path merge into one array
- **WHEN** the user passes `-f items.0.id=abc -f items.1.id=def`
- **THEN** the request body SHALL be `{"items": [{"id": "abc"}, {"id": "def"}]}`

#### Scenario: Integer segment nested under dot-path object key
- **WHEN** the user passes `-f relationships.workspaces.data.0.type=workspaces -f relationships.workspaces.data.0.id=<uuid>`
- **THEN** the request body SHALL be `{"relationships": {"workspaces": {"data": [{"type": "workspaces", "id": "<uuid>"}]}}}`
