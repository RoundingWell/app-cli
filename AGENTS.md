# AGENTS.md
- Do not preserve backward compatibility. Remove obsolete paths instead of adding compatibility layers, fallbacks, or migrations.
- Choose the simplest implementation that fully meets the current requirements. Avoid speculative abstractions, configuration, and indirection.
- Grow the system in layers. Start from the smallest version that works end to end, and add each new capability on top of a product that already works. Never trade a working product for unfinished complexity.
- Keep components modular and concerns clearly separated.
- Prefer established, well-maintained libraries when they reduce overall complexity or improve reliability. Do not reimplement common functionality without a clear reason.
- Lean on the dependencies already in the project before writing your own implementation or adding packages. Do not assume a library lacks a capability without checking its documentation and types.
- Make architectural decisions for the long term. Do not accept a stopgap that only works for now and is meant to be replaced later.
- Use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) when writing commit messages.
- Use test-driven development (TDD) practices.
- Update documentation (`README.md`, `CONTRIBUTING.md`, any relevant files in `docs/`) when updating code.
- Update `skills/rw-skill.md` when adding, removing, or modifying any `rw` command or flag (except: `auth`, `api`, `update`, or `config`).
- Only update `CHANGELOG.md` when drafting a release. Do not update the changelog at any other time. Skip `ci` and `build` commits when updating the changelog. Keep the compare links up to date. Call out breaking changes with a warning ⚠️ symbol.
- Use semantic versioning for release tags. Never include a `v` prefix on versions. Always sign release tags.

Documentation of `rw` configuration files is in [docs/config.md](./docs/config.md).
Documentation of the RoundingWell API is in [docs/json-api.md](./docs/json-api.md).

### Common Commands

```sh
cargo build --release                                       # Build (binary output: target/release/rw)
cargo install --path .                                      # Install to ~/.cargo/bin/rw
cargo test                                                  # Run all tests
cargo test <name>                                           # Run a single test by name
cargo fmt                                                   # Format
cargo fmt --check                                           # Verify formatting
cargo clippy --all-targets --all-features -- -D warnings    # Lint
cargo audit                                                 # Audit dependencies for security advisories
```

- Run `cargo clippy` and `cargo fmt` after changes
- Run `cargo test` before `cargo build`
