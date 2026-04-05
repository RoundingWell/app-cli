## MODIFIED Requirements

### Requirement: Key path conflict is reported as an error
The system SHALL return a non-zero exit status and emit an error when a dot-path key conflicts with an already-set value. The error message SHALL be `"Unable to set field {key} because it conflicts with another field"`, where `{key}` is the full dot-path key that could not be set.

#### Scenario: Conflicting flat and nested keys
- **WHEN** the user passes `-f foo=bar -f foo.baz=qux`
- **THEN** the command SHALL exit with a non-zero status and print the error `"Unable to set field foo.baz because it conflicts with another field"`

#### Scenario: Multi-level conflict reports the full key
- **WHEN** the user passes `-f a.b=x -f a.b.c=y`
- **THEN** the command SHALL exit with a non-zero status and print the error `"Unable to set field a.b.c because it conflicts with another field"`
