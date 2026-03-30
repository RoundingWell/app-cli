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

- Prefer `rg` to `grep` and `find` when searching files
- Run `cargo fmt` after changes
- Run `cargo test` before `cargo build`
- Use test driven development (TDD) principles; create tests that fail, then implement to satisfy test
- Prefer unit tests to integration tests, mock network boundaries
- The API always sends and receives data in [JSON:API format](https://jsonapi.org/) (e.g. `{ "data": { "type": "clinicians", "id": "<uuid>", "attributes": { ... } } }`)

## Git Tags

- Always follow [semantic versioning](https://semver.org/) for tagging versions
- Never use a "v" prefix for tagging versions (e.g. use `1.2.3` not `v1.2.3`)

## Git Commits

- Use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary) (CC) when making commits
- Use branch names that start with a CC type like `feat-`, `fix-`, `ci-`, `chore-`, etc

## Pull Requests

- Add a Shortcut story reference to the pull request body, like `[sc-1234]`

## Shortcut Stories

- When creating Shortcut stories for this project:
  * The team should be (the id of) "Backend"
  * The workflow should be (the id of) "Development"
  * The type should be (the id of) either "Feature", "Bug", or "Chore"; use Chore when neither Feature or Bug makes sense
  * When the type is "Chore", set the Chore Type (custom field) to "Improvement"
  * The epic should be choosen by year + month + type; for example:
    a Feature story created in March 2026 would be "Mar '26 Improvements",
    a Bug story created in July 2026 would be "Jul '26 Bugs"
  * When no matching epic exists by year + month, create the epic before the story
