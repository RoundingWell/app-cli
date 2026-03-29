                   .-==++++++=-:.                 
               :=*####****########*=:             
            .=##+-:.         .-+######=.          
          .+#+:                  .=#####+.        
         =#=                        -#####=       
        ++.                          .*####*      
       +=                              +####*     
      :+                                *####+    
      +                                 -#####.   
      =                                 .#####=   
      .                 .               .#####+   
                      :=                -#####+   
                    :*-                 ######-   
                  :*#.                 +######    
                :*#+                 .*######-    
              :*##-                .=#######+     
            :*###*:              -+########=      
          :*########*+=-::::-=+###########:       
        :*##############################=         
         -*##########################*-           
           .-+####################+-.             
               .:=++*######**+=-.                 
                  ROUNDING_WELL

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

| Flag        | Short | Description               |
|-------------|-------|---------------------------|
| `--profile` | `-p`  | Named profile to use      |
| `--json`    |       | Change all output to JSON |

All commands require a profile. Set a default with `rw profile <name>`, or pass `--profile` on each invocation.
See [Adding a profile](#adding-a-profile) to add a profile.

Stage-to-domain mapping:

| Stage     | Domain                                            |
|-----------|---------------------------------------------------|
| `prod`    | `https://{organization}.roundingwell.com`         |
| `sandbox` | `https://{organization}-sandbox.roundingwell.com` |
| `qa`      | `https://{organization}.roundingwell.com`         |
| `dev`     | `https://{organization}.roundingwell.dev`         |
| `local`   | `http://localhost:8080`                           |

### Profiles

```sh
rw profiles                 # List all configured profiles
rw profiles add mercy       # Add the "mercy" profile (see below)
rw profiles rm mercy        # Remove the "mercy" profile
rw profile mercy            # Set "mercy" as the default profile (profile must already exist)
```

#### Adding a profile

```sh
# Interactive – prompts for organization and stage
rw profiles add mercy

# Non-interactive – provide all values as flags
rw profiles add mercy --organization mercy-clinic --stage prod

# Short flags
rw profiles add mercy -o mercy-clinic -g prod

# JSON output (requires --organization and --stage; errors if either is missing)
rw profiles add mercy -o mercy-clinic -g prod --json
```

| Flag             | Short | Description                                             |
|------------------|-------|---------------------------------------------------------|
| `--organization` | `-o`  | Organization slug (e.g. `mercy-clinic`)                 |
| `--stage`        | `-g`  | Stage: `prod`, `sandbox`, `qa`, `dev`, or `local`       |

If either flag is omitted, the CLI prompts for the missing value interactively. Passing `--json` without both flags is an error.

### Authentication

```sh
rw auth login              # Open browser and authenticate via WorkOS
rw auth status             # Show authentication status for current profile
rw auth status --show      # Also print the stored token or credentials
rw auth logout             # Remove stored credentials for current profile

# Use a named profile
rw auth login --profile mercy
```

### API requests

```sh
# GET /api/clinicians (default profile)
rw api clinicians

# POST with JSON body fields
rw api clinicians --method POST --field name="Alice Avalon" --field email=alice.avalon@mercy.org

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

See [CONTRIBUTING.md](CONTRIBUTING.md) for development instructions.
