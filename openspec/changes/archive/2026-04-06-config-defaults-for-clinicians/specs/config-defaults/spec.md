## ADDED Requirements

### Requirement: Set a config default
The `config default set <key> <value>` subcommand SHALL write `value` into the `default` map of the active profile under `key`. Only the keys `team` and `role` SHALL be accepted.

#### Scenario: Set a valid default key
- **WHEN** the user runs `rw config default set role employee`
- **THEN** the CLI SHALL update `default.role` in the active profile config to `"employee"` and print a confirmation

#### Scenario: Set another valid default key
- **WHEN** the user runs `rw config default set team NUR`
- **THEN** the CLI SHALL update `default.team` in the active profile config to `"NUR"` and print a confirmation

#### Scenario: Overwrite an existing default
- **WHEN** `default.role` is already set and the user runs `rw config default set role physician`
- **THEN** the CLI SHALL overwrite the existing value and print a confirmation

#### Scenario: Reject an unknown key
- **WHEN** the user runs `rw config default set foo bar`
- **THEN** the CLI SHALL exit with a non-zero status and print an error listing the allowed keys (`team`, `role`)

### Requirement: Get a config default
The `config default get <key>` subcommand SHALL print the current value of `key` in the `default` map of the active profile. If the key is not set, it SHALL exit with a zero status and produce no output. Only the keys `team` and `role` SHALL be accepted.

#### Scenario: Get a key that is set
- **WHEN** `default.role` is `"employee"` and the user runs `rw config default get role`
- **THEN** the CLI SHALL print `employee`

#### Scenario: Get a key that is not set
- **WHEN** `default.team` is not set and the user runs `rw config default get team`
- **THEN** the CLI SHALL exit with a zero status and produce no output

#### Scenario: Reject an unknown key
- **WHEN** the user runs `rw config default get foo`
- **THEN** the CLI SHALL exit with a non-zero status and print an error listing the allowed keys (`team`, `role`)

### Requirement: Remove a config default
The `config default rm <key>` subcommand SHALL remove `key` from the `default` map of the active profile. If the key is not set, the command SHALL succeed silently. Only the keys `team` and `role` SHALL be accepted.

#### Scenario: Remove a key that is set
- **WHEN** `default.role` is set and the user runs `rw config default rm role`
- **THEN** the CLI SHALL remove the key from the profile config and print a confirmation

#### Scenario: Remove a key that is not set
- **WHEN** `default.team` is not set and the user runs `rw config default rm team`
- **THEN** the CLI SHALL exit with a zero status without printing an error

#### Scenario: Reject an unknown key
- **WHEN** the user runs `rw config default rm foo`
- **THEN** the CLI SHALL exit with a non-zero status and print an error listing the allowed keys (`team`, `role`)
