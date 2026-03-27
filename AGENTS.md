# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build --release   # Build (binary output: target/release/rw)
cargo install --path .  # Install to ~/.cargo/bin/rw
cargo test              # Run all tests
cargo test <name>       # Run a single test by name (substring match)
cargo clippy            # Lint
cargo fmt               # Format
```

## Architecture

This is a Rust CLI (`rw`) built with `clap` (derive API) and `tokio` async runtime. There are five modules:

- **`cli`** — Clap structs for all CLI arguments and subcommands. The `Stage` enum and `validate_slug` parser live here.
- **`config`** — Reads/writes `~/.config/rw/profiles.json`. Defines `Config`, `Profile`, `AuthEntry` (bearer or basic), and `resolve_profile` (resolves the active profile name, org, and stage).
- **`api`** — Single function `resolve_api` mapping `(org, stage)` → base URL.
- **`commands/auth`** — Implements `rw auth login/status/logout`. Login uses the OAuth Device Authorization Flow via WorkOS AuthKit: requests a device code, opens the browser, polls for a token, and persists the bearer token in config.
- **`commands/api`** — Implements `rw api <endpoint>`. Looks up auth credentials, builds an HTTP request (optional JSON body from `--field` pairs), and pretty-prints the response. Supports `--jq` to filter output by piping through the `jq` binary.

**Data flow:** `main.rs` parses CLI → dispatches to command → command calls `resolve_profile` when it needs org/stage. `Commands::Profile` and `Commands::Profiles` operate directly on config without resolving a profile.

**Profiles:** Profiles are named entries in `config.profiles` mapping a name → `(organization, stage)`. `rw profile <name>` creates the profile interactively if it doesn't exist, then sets it as `default_profile`. All subcommands accept `--profile` to override the default.

**Authentication storage:** Credentials in `config.authentication` are keyed by **profile name** (not org slug). Bearer tokens come from `rw auth login`. Basic credentials must be added manually to `~/.config/rw/profiles.json`.

**WorkOS config:** Two WorkOS environments are used. `Stage::Prod` and `Stage::Sandbox` use the production AuthKit tenant; `Stage::Qa`, `Stage::Dev`, and `Stage::Local` use the dev tenant.

**API URL construction:** Built by `api::resolve_api(org, stage)`. Local stage hits `localhost:8080` directly; Dev uses `.roundingwell.dev/api`; all others use `.roundingwell.com/api` (Sandbox appends `-sandbox` to the subdomain).
