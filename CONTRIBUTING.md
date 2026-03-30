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
