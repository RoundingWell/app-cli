# AGENTS.md

This file provides guidance to AI agents when working with code in this repository.

## Commands

```sh
cargo build --release   # Build (binary output: target/release/rw)
cargo install --path .  # Install to ~/.cargo/bin/rw
cargo test              # Run all tests
cargo test <name>       # Run a single test by name (substring match)
cargo clippy            # Lint
cargo fmt               # Format
```

## Instructions

- Run `cargo fmt` after changes
- Run `cargo test` before `cargo build`
- Use test driven development (TDD) principles; create tests that fail, then implement to satisfy test
- Prefer unit tests to integration tests, mock network boundaries
