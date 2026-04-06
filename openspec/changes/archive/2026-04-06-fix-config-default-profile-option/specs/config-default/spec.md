## MODIFIED Requirements

### Requirement: Set a config default
The `config default set <key> <value>` subcommand SHALL write `value` into the `default` map of the target profile under `key`. The target profile is the one named by the global `--profile` option when supplied, otherwise the configured default profile. Only the keys `team` and `role` SHALL be accepted.

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

#### Scenario: Set a default on a named profile
- **WHEN** the user runs `rw --profile staging config default set role physician`
- **THEN** the CLI SHALL update `default.role` in the `staging` profile config to `"physician"` and print a confirmation, leaving all other profiles unchanged

### Requirement: Get a config default
The `config default get <key>` subcommand SHALL print the current value of `key` in the `default` map of the target profile. The target profile is the one named by the global `--profile` option when supplied, otherwise the configured default profile. If the key is not set, it SHALL exit with a zero status and produce no output. Only the keys `team` and `role` SHALL be accepted.

#### Scenario: Get a key that is set
- **WHEN** `default.role` is `"employee"` and the user runs `rw config default get role`
- **THEN** the CLI SHALL print `employee`

#### Scenario: Get a key that is not set
- **WHEN** `default.team` is not set and the user runs `rw config default get team`
- **THEN** the CLI SHALL exit with a zero status and produce no output

#### Scenario: Reject an unknown key
- **WHEN** the user runs `rw config default get foo`
- **THEN** the CLI SHALL exit with a non-zero status and print an error listing the allowed keys (`team`, `role`)

#### Scenario: Get a default from a named profile
- **WHEN** the `staging` profile has `default.role` set to `"nurse"` and the user runs `rw --profile staging config default get role`
- **THEN** the CLI SHALL print `nurse`, regardless of what the active profile's `default.role` is

### Requirement: Remove a config default
The `config default rm <key>` subcommand SHALL remove `key` from the `default` map of the target profile. The target profile is the one named by the global `--profile` option when supplied, otherwise the configured default profile. If the key is not set, the command SHALL succeed silently. Only the keys `team` and `role` SHALL be accepted.

#### Scenario: Remove a key that is set
- **WHEN** `default.role` is set and the user runs `rw config default rm role`
- **THEN** the CLI SHALL remove the key from the profile config and print a confirmation

#### Scenario: Remove a key that is not set
- **WHEN** `default.team` is not set and the user runs `rw config default rm team`
- **THEN** the CLI SHALL exit with a zero status without printing an error

#### Scenario: Reject an unknown key
- **WHEN** the user runs `rw config default rm foo`
- **THEN** the CLI SHALL exit with a non-zero status and print an error listing the allowed keys (`team`, `role`)

#### Scenario: Remove a default from a named profile
- **WHEN** the `staging` profile has `default.team` set and the user runs `rw --profile staging config default rm team`
- **THEN** the CLI SHALL remove `default.team` from the `staging` profile and print a confirmation, leaving all other profiles unchanged

### Requirement: List config defaults
The `config default list` subcommand SHALL print all defined default key/value pairs from the target profile's `default` map, one per line in `<key>=<value>` format. The target profile is the one named by the global `--profile` option when supplied, otherwise the configured default profile. If no defaults are set, it SHALL exit with a zero status and produce no output.

#### Scenario: List when defaults are set
- **WHEN** `default.role` is `"employee"` and `default.team` is `"NUR"` and the user runs `rw config default list`
- **THEN** the CLI SHALL print each key/value pair on its own line (e.g., `role=employee` and `team=NUR`) and exit with a zero status

#### Scenario: List when no defaults are set
- **WHEN** no defaults are configured and the user runs `rw config default list`
- **THEN** the CLI SHALL exit with a zero status and produce no output

#### Scenario: List when only one default is set
- **WHEN** `default.role` is `"physician"` and `default.team` is not set and the user runs `rw config default list`
- **THEN** the CLI SHALL print only `role=physician` and exit with a zero status

#### Scenario: List defaults for a named profile
- **WHEN** the `staging` profile has `default.role` set to `"nurse"` and the user runs `rw --profile staging config default list`
- **THEN** the CLI SHALL print the defaults from the `staging` profile, regardless of the active profile's defaults
