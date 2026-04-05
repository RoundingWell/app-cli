## MODIFIED Requirements

### Requirement: Key path conflict is reported as an error
The system SHALL return a clear error when a dot-path key conflicts with an already-set leaf value (i.e. one field sets `foo` as a leaf and another attempts to descend into `foo.<child>`). The error message SHALL reference the conflicting prefix segment (the first dot-path component that collides), not the full descending key.

#### Scenario: Conflicting flat and nested keys
- **WHEN** the user passes `-f foo=bar -f foo.baz=qux`
- **THEN** the command SHALL exit with a non-zero status and print an error with the message `"field key conflict: 'foo' is both a leaf and a nested path"`
