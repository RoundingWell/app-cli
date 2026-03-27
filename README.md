# app-cli

RoundingWell Command Line Interface — `rw`

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

## Configuration

`rw` stores credentials and profiles in `~/.config/rw/profiles.json`.
The file is created automatically on first use.

Example configuration:

```json
{
  "profiles": {
    "demo": {
      "organization": "demonstration",
      "stage": "prod"
    },
    "woody": {
      "organization": "woody",
      "stage": "dev"
    }
  },
  "authentication": {
    "demonstration": {
      "bearer": "<jwt-token>"
    },
    "woody": {
      "basic": {
        "username": "woody.gilk@roundingwell.com",
        "password": "<plaintext-password>"
      }
    }
  }
}
```

## Usage

### Global options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--organization` | `-o` | `demonstration` | Organization slug |
| `--stage` | `-s` | `prod` | Stage: `prod`, `sandbox`, `qa`, `dev` |
| `--profile` | `-p` | — | Named profile (overrides `--organization` / `--stage`) |

Stage-to-domain mapping:

| Stage | Domain |
|-------|--------|
| `prod`, `qa` | `https://{org}.roundingwell.com` |
| `dev` | `https://{org}.roundingwell.dev` |
| `sandbox` | `https://{org}-sandbox.roundingwell.com` |

### Authentication

```sh
rw auth login              # Open browser and authenticate via WorkOS
rw auth status             # Show stored credential status
rw auth logout             # Remove stored credentials

# Authenticate against a specific organization / stage
rw auth login --organization myorg --stage dev

# Use a named profile
rw auth login --profile woody
```

### API requests

```sh
# GET /api/clinicians (default org + stage)
rw api clinicians

# GET with a different organization
rw api clinicians --organization myorg

# POST with JSON body fields
rw api clinicians --method POST --field name=Alice --field role=clinician

# Add extra request headers
rw api clinicians --header "Accept: application/json"

# Print raw (unpretty) JSON
rw api clinicians --raw

# Use a named profile
rw api clinicians --profile demo
```

## Development

Run the test suite:

```sh
cargo test
```

Check for lint warnings:

```sh
cargo clippy
```

Format code:

```sh
cargo fmt
```

