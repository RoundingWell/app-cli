# Contributing

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

| Path                                            | Contents                                            |
|-------------------------------------------------|-----------------------------------------------------|
| `~/.config/rw/config.json`                      | Tool configuration                                  |
| `~/.config/rw/auth/{organization}-{stage}.json` | Auth credentials per organization+stage (mode 0600) |

### config.json

```json
{
  "default": "demo",
  "profiles": {
    "demo": {
      "organization": "demonstration",
      "stage": "prod"
    },
    "mercy": {
      "organization": "mercy",
      "stage": "dev"
    }
  }
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
