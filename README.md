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

### Global options

| Flag           | Short | Description               |
|----------------|-------|---------------------------|
| `--profile`    | `-p`  | Named profile to use      |
| `--config-dir` | `-c`  | Configuration directory   |
| `--json`       |       | Change all output to JSON |

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

### Authentication

```sh
rw auth login       # Open browser and authenticate via WorkOS
rw auth status      # Show authentication status for current profile
rw auth header      # Show the authentication header for current profile
rw auth logout      # Remove stored credentials for current profile

# Use a named profile
rw auth login --profile mercy
```

### Basic Auth

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

### Clinicians

```sh
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

### API requests

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

Stage-to-domain mapping:

| Stage     | Domain                                            |
|-----------|---------------------------------------------------|
| `prod`    | `https://{organization}.roundingwell.com`         |
| `sandbox` | `https://{organization}-sandbox.roundingwell.com` |
| `qa`      | `https://{organization}.roundingwell.com`         |
| `dev`     | `https://{organization}.roundingwell.dev`         |
| `local`   | `http://localhost:8080`                           |

### Updates

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
