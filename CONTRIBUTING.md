# Contributing

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (includes `cargo`)

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

## Project layout

```
src/
├── main.rs            entry + dispatch
├── cli.rs             clap argument structs + Stage + slug validation
├── config.rs          Config / Profile / AppContext + on-disk persistence
├── api.rs             stage → API URL resolution
├── auth_cache.rs      AuthCache (Bearer | Basic) + 0600 on-disk store
├── http.rs            ApiClient: auth-attached reqwest wrapper
├── jsonapi.rs         generic Document / Resource / Single / List envelopes
├── prompt.rs          interactive yes_no / text / stage / organization prompts
├── output.rs          Output{json} + CommandOutput trait
├── migration.rs       one-shot config migrations on startup
├── version_check.rs   GitHub release check + self_update
└── commands/          one module per top-level subcommand
```

`http.rs` and `jsonapi.rs` are the canonical way to reach the API. New
commands should compose them rather than reaching for `reqwest::Client`
directly:

```rust
let api = ApiClient::new(ctx).await?;
let teams: List<TeamAttributes> = api.get("teams").await?;
```

`prompt::*_with` variants take generic `Read`/`Write` and are used directly
in unit tests; the bare `prompt::yes_no` / `prompt::text` / etc. wrappers
default to stdin and stderr.
