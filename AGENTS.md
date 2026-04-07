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

- Use short sentences, avoid preamble and filler
- Display tools and results, avoid explanation
- Use test driven development (TDD) approach; first create tests that fail, then implement to satisfy tests
- Always update docs (README.md, CONTRIBUTING.md, any relevant files in `docs/`) when adding or modifying commands
- Always mock network boundaries in tests
- Run `cargo clippy` and `cargo fmt` after changes
- Run `cargo test` before `cargo build`

## Important Notes 

- The API always sends and receives data in [JSON:API format](https://jsonapi.org/).

### JSON:API Examples

A single resource response example:

```json
{
  "data": {
    "type": "clinicians",
    "id": "<uuid>",
    "attributes": {
      "name": "<string>",
      "...": "..."
    },
    "relationships": {
      "team": {
        "data": {"type": "teams", "id": "<uuid>"}
      },
      "workspaces": {
        "data": [
          {"type": "workspaces", "id": "<uuid>"},
          {"type": "workspaces", "id": "..."}
        ]
      }
    }
  }
}
```

A multiple resource response example:

```json
{
  "data": [
    {
      "type": "workspaces",
      "id": "<uuid>",
      "attributes": { "...": "..." },
      "relationships": { "...": "..." }
    }
  ]
}
```

## Changelog

- Only update CHANGELOG.md when drafting a release
- Call out BREAKING changes with a warning ⚠️ symbol
- Skip `ci` and `build` commits
- Update compare links at the bottom of the file

## Git Tags

- Always follow [semantic versioning](https://semver.org/) for tagging versions
- Never use a "v" prefix for tagging versions (e.g. use `1.2.3` not `v1.2.3`)

## Git Commits

- Use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary) (CC) when making commits
- Use branch names that start with a CC type like `feat-`, `fix-`, `ci-`, `chore-`, etc

## Pull Requests

- When creating a PR, do not create it as a draft
- When merging a PR, prefer merging over squashing
