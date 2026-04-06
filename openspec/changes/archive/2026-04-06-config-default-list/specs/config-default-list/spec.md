## ADDED Requirements

### Requirement: List config defaults
The `config default list` subcommand SHALL print all defined default key/value pairs from the active profile's `default` map, one per line in `<key>=<value>` format. If no defaults are set, it SHALL exit with a zero status and produce no output.

#### Scenario: List when defaults are set
- **WHEN** `default.role` is `"employee"` and `default.team` is `"NUR"` and the user runs `rw config default list`
- **THEN** the CLI SHALL print each key/value pair on its own line (e.g., `role=employee` and `team=NUR`) and exit with a zero status

#### Scenario: List when no defaults are set
- **WHEN** no defaults are configured and the user runs `rw config default list`
- **THEN** the CLI SHALL exit with a zero status and produce no output

#### Scenario: List when only one default is set
- **WHEN** `default.role` is `"physician"` and `default.team` is not set and the user runs `rw config default list`
- **THEN** the CLI SHALL print only `role=physician` and exit with a zero status
