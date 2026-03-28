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

## Usage

### Global options

| Flag        | Short | Description          |
|-------------|-------|----------------------|
| `--profile` | `-p`  | Named profile to use |

All commands require a profile. Set a default with `rw profile <name>`, or pass `--profile` on each invocation.

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
rw profile mercy       # Set "mercy" as the default profile
rw profiles            # List all configured profiles
```

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
