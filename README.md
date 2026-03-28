# RoundingWell CLI

The `rw` command line interface for [RoundingWell](https://www.roundingwell.com/).

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (includes `cargo`)
- OpenSSL development libraries (Linux only — usually pre-installed on macOS/Windows)

```sh
# Debian / Ubuntu
sudo apt install pkg-config libssl-dev

# Fedora / RHEL
sudo dnf install pkg-config openssl-devel
```

## Building from source

```sh
git clone https://github.com/RoundingWell/app-cli.git
cd app-cli
cargo build --release
```

The compiled binary is placed at `target/release/rw`.

## Installing

### Install into `~/.cargo/bin` (recommended)

```sh
cargo install --path .
```

This places `rw` on your `$PATH` automatically (assuming `~/.cargo/bin` is in your `PATH`).

### Manual install

Copy the binary to any directory on your `$PATH`:

```sh
# macOS / Linux
cp target/release/rw /usr/local/bin/rw
```

Verify the installation:

```sh
rw --version
```

## Storage

`rw` stores files under `~/.config/rw/`:

| Path                                            | Contents                                                  |
|-------------------------------------------------|-----------------------------------------------------------|
| `~/.config/rw/profiles.json`                    | Named profiles (organization + stage) and default profile |
| `~/.config/rw/auth/{organization}-{stage}.json` | Auth credentials per organization+stage (mode 0600)       |

### profiles.json

```json
{
  "profiles": {
    "demo": {
      "organization": "demonstration",
      "stage": "prod"
    },
    "mercy": {
      "organization": "mercy",
      "stage": "dev"
    }
  },
  "default_profile": "demo"
}
```

### auth/{organization}-{stage}.json

Bearer token (written after `rw auth login`):

```json
{
  "access_token": "<jwt>",
  "refresh_token": "<token>",
  "expires_at": 1234567890
}
```

Basic credentials (written manually):

```json
{
  "username": "jane.doe@roundingwell.com",
  "password": "<plaintext-password>"
}
```

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

## Development

Format code:

```sh
cargo fmt
```

Check for lint warnings:

```sh
cargo clippy
```

Run the test suite:

```sh
cargo test
```
