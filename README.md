<img src="docs/rw-cli.png" width="300" alt="RW_CLI"/>

# RoundingWell CLI

The `rw` command line interface for [RoundingWell](https://www.roundingwell.com/).

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/RoundingWell/app-cli/main/install.sh | bash
```

**Options** (via environment variables):

| Variable      | Description                                  | Default         |
|---------------|----------------------------------------------|-----------------|
| `RW_BIN_DIR`  | Where to install the binary                  | `~/.local/bin`  |
| `RW_VERSION`  | Specific version to install                  | latest          |

```sh
# Install a specific version
RW_VERSION=1.2.3 bash -c "$(curl -fsSL https://raw.githubusercontent.com/RoundingWell/app-cli/main/install.sh)"
```

## Usage

| Flag           | Short | Description                                                            |
|----------------|-------|------------------------------------------------------------------------|
| `--profile`    | `-p`  | Named profile to use                                                   |
| `--auth`       | `-A`  | Use stored credentials from another profile (overrides only the auth) |
| `--config-dir` | `-c`  | Configuration directory                                                |
| `--json`       |       | Change all output to JSON                                              |

All commands require a profile. Set a default with `rw config profile use <name>`, or pass `--profile` on each invocation.
See [Adding a profile](#adding-a-profile) to add a profile.

### Profiles

```sh
rw config profile list                  # List all configured profiles
rw config profile show                  # Show the active profile
rw config profile use mercy             # Set "mercy" as the default profile
rw config profile add mercy             # Add the "mercy" profile (see below)
rw config profile rm mercy              # Remove the "mercy" profile (prompts for confirmation)
rw config profile rm mercy --yes        # Remove without prompting
rw config profile set mercy -o new-org  # Update organization for a profile
rw config profile set mercy -g sandbox  # Update stage for a profile
rw config profile auth mercy            # Save basic auth credentials for a profile (see below)
```

#### Adding a profile

```sh
# Interactive – prompts for organization and stage
rw config profile add mercy

# Non-interactive – provide all values as flags
rw config profile add mercy --organization mercy-clinic --stage prod

# Short flags
rw config profile add mercy -o mercy-clinic -g prod

# Set as default after creation
rw config profile add mercy -o mercy-clinic -g prod --use

# JSON output (requires --organization and --stage; errors if either is missing)
rw config profile add mercy -o mercy-clinic -g prod --json
```

| Flag             | Short | Description                                             |
|------------------|-------|---------------------------------------------------------|
| `--organization` | `-o`  | Organization slug (e.g. `mercy-clinic`)                 |
| `--stage`        | `-g`  | Stage: `prod`, `sandbox`, `qa`, `dev`, or `local`       |
| `--use`          |       | Set new profile as default after creation               |

If either `--organization` or `--stage` is omitted, the CLI prompts for the missing value interactively. Passing `--json` without both flags is an error.

#### Profile Defaults

Per-profile default values can be set for keys that commands use internally. Currently supported keys: `role`, `team`.

```sh
rw config default set role physician   # Set default role to "physician"
rw config default set team ICU         # Set default team abbreviation to "ICU"
rw config default get role             # Print current default role (no output if unset)
rw config default get team             # Print current default team
rw config default rm role              # Remove default role
rw config default rm team              # Remove default team
rw config default list                 # List all defined defaults (key=value, alphabetically)
```

_**Note:** When a default is not set, commands fall back to their built-in values._

#### Using Basic Auth

```sh
rw config profile auth mercy                        # Store basic auth credentials (prompted interactively)
rw config profile auth mercy --username alice       # Password prompted securely
rw config profile auth mercy --username alice \
  --password secret                                 # Fully non-interactive
```

| Flag         | Short | Description             |
|--------------|-------|-------------------------|
| `--username` | `-u`  | Username for basic auth |
| `--password` | `-P`  | Password for basic auth |

#### Diagnostics

`rw config doctor` runs a fixed sequence of checks against the active profile (or `--profile`):

```sh
rw config doctor          # Plain checklist
rw config doctor --json   # Structured report
```

The four checks are:

| Check      | What it verifies                                                  |
|------------|-------------------------------------------------------------------|
| `profile`  | A profile is selected and resolves to an organization + stage     |
| `auth`     | Stored credentials exist and (for bearer tokens) aren't expired   |
| `api`      | `GET /clinicians/me` returns a 2xx, with status + latency reported |
| `defaults` | Lists configured `rw config default` keys (informational)         |

Each check reports `pass`, `warn`, `fail`, `skip`, or `info`. A later check is skipped when an earlier one it depends on fails. The command exits non-zero if any check fails.

### Authentication

```sh
rw auth login       # Open browser and authenticate via WorkOS
rw auth status      # Show authentication status for current profile
rw auth header      # Show the authentication header for current profile
rw auth logout      # Remove stored credentials for current profile

# Use a named profile
rw auth login --profile mercy
```

#### Auth override

Use `--auth` (`-A`) to borrow stored credentials from another profile without
changing the active profile's organization, stage, or defaults. This is
useful when you have multiple identities for the same environment (for
example, a personal account plus a service account).

```sh
# Run a request against the active profile's environment, but as the
# user whose credentials are saved under "service".
rw clinicians show me --auth service

# Combine with --profile: target profile A's environment, send with
# profile B's credentials.
rw api ping --profile demo --auth service
```

`--auth` is rejected on `rw auth login` and `rw auth logout` (those
commands inherently target a specific profile). It is permitted on
`rw auth status` and `rw auth header`, where it reports on the override
profile's credentials.

## Tools

### Actions

```sh
# Trace an action's patient, program, and form workspaces to find misalignments
rw actions trace 60fda0c4-eca0-434a-80d8-fd4e490aa051

# Output as JSON
rw actions trace 60fda0c4-eca0-434a-80d8-fd4e490aa051 --json
```

The command resolves the action, then fetches the related patient, program, and form. It renders one workspace tree per resource and reports any workspace memberships where the patient does not align with the program (via the action) or with the form.

### Artifacts

```sh
# List artifacts filtered by type, path, and search term
rw artifacts list <type> --path=<path> --term=<term>

# Output as JSON
rw artifacts list <type> --path=<path> --term=<term> --json
```

### Clinicians

```sh
# Show a clinician (by UUID, email, or "me")
rw clinicians show joe@example.com
rw clinicians show 60fda0c4-eca0-434a-80d8-fd4e490aa051
rw clinicians show me

# Enable or disable a clinician (by UUID or email)
rw clinicians enable joe@example.com
rw clinicians disable 60fda0c4-eca0-434a-80d8-fd4e490aa051

# Assign a clinician (by UUID or email) to a team (by UUID or abbr)
rw clinicians assign joe@example.com NUR
rw clinicians assign joe@example.com 60c0e3b8-64b6-491f-a502-7346d14b3192

# Grant a clinician (by UUID or email) a role (by UUID or name)
rw clinicians grant joe@example.com admin
rw clinicians grant joe@example.com 60c0e3b8-64b6-491f-a502-7346d14b3192

# Prepare a clinician: assigns the correct role, team, visibility, and default workspaces
rw clinicians prepare joe@example.com
rw clinicians prepare 60fda0c4-eca0-434a-80d8-fd4e490aa051

# Register a new clinician
rw clinicians register joe@example.com "Joe Smith"
rw clinicians register joe@example.com "Joe Smith" --role employee
rw clinicians register joe@example.com "Joe Smith" --team NUR
rw clinicians register joe@example.com "Joe Smith" --role employee --team NUR

# Update a clinician attribute (by UUID, email, or "me")
rw clinicians update joe@example.com --field name --value "Jane Doe"
rw clinicians update 60fda0c4-eca0-434a-80d8-fd4e490aa051 --field email --value jane@example.com
rw clinicians update me --field npi --value 1234567890
rw clinicians update me --field credentials --value "RN,MD"
rw clinicians update me --field npi          # omit --value to clear (sends null)
rw clinicians update me --field credentials  # omit --value to clear (sends [])

# Use a named profile
rw clinicians enable joe@example.com --profile mercy
```

### Roles

```sh
# List all roles (sorted by label)
rw roles list

# Output as JSON
rw roles list --json

# Show a role by UUID or name (includes id, name, label, description, permissions)
rw roles show <uuid-or-name>

# Output as JSON
rw roles show <uuid-or-name> --json
```

### Teams

```sh
# List all teams (sorted by abbreviation)
rw teams list

# Output as JSON
rw teams list --json

# Show a team by UUID or abbreviation
rw teams show <uuid-or-abbr>

# Output as JSON
rw teams show <uuid-or-abbr> --json
```

### Workspaces

```sh
# List all workspaces (sorted by name)
rw workspaces list

# Output as JSON
rw workspaces list --json

# Show a workspace by UUID or slug
rw workspaces show cardiology
rw workspaces show 11111111-1111-1111-1111-111111111111

# Show as JSON
rw workspaces show cardiology --json
```

## API requests

```sh
# GET /api/clinicians (default profile)
rw api clinicians

# POST with JSON body fields using dot-path keys
rw api clinicians --method POST --field data.attributes.name="Alice Avalon" --field data.attributes.email=alice.avalon@mercy.org --field data.relationships.role.data.type=roles --field data.relationships.role.data.id=<uuid>

# Add extra request headers
rw api clinicians --header "Accept: application/json"

# Filter output with a jq expression
rw api clinicians --jq '.[0]'

# Print raw (unpretty) JSON
rw api clinicians --raw

# Use a named profile
rw api clinicians --profile demo
```

_**Note**: jq expression filtering uses [jaq](https://github.com/01mf02/jaq), which may have slight differences
in formatting and may not support all jq features._

Stage-to-base-URL mapping (the endpoint you pass is appended to this):

| Stage     | Base URL                                              |
|-----------|-------------------------------------------------------|
| `prod`    | `https://{organization}.roundingwell.com/api`         |
| `sandbox` | `https://{organization}-sandbox.roundingwell.com/api` |
| `qa`      | `https://{organization}.roundingwell.com/api`         |
| `dev`     | `https://{organization}.roundingwell.dev/api`         |
| `local`   | `http://localhost:8080`                               |

## Agent Skills

Install a skill that teaches Claude Code how to use `rw`:

```sh
rw skills install               # Install to ~/.claude/skills/rw/SKILL.md (global)
rw skills install --local       # Install to .claude/skills/rw/SKILL.md in the current directory
rw skills install --no-clobber  # Skip if the skill file already exists
```

| Flag           | Description                                       |
|----------------|---------------------------------------------------|
| `--local`      | Write to `.claude/` instead of global `~/.claude` |
| `--no-clobber` | Do not overwrite an existing skill file           |

## Updates

```sh
rw update   # Update rw to the latest version
```

When a newer version is detected while running any command, the CLI will prompt once (on interactive terminals) whether to enable automatic updates:

```
A new version of rw is available: 0.4.0 (you have 0.3.1)
Enable automatic updates? [y/N]
```

The preference is stored as `auto_update` in `~/.config/rw/config.json`. Once set, no further prompting occurs — the CLI either updates silently before each command (if enabled) or shows a warning (if disabled). Automatic updates can also be managed with:

```sh
rw config updates show     # Show current update setting
rw config updates enable   # Enable automatic updates
rw config updates disable  # Disable automatic updates
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development instructions.
