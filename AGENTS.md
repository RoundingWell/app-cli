# AGENTS.md

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 5. Development Guidelines

- Always use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) when writing commit messages.
- Always use test-driven development (TDD) practices.
- Always update documentation (`README.md`, `CONTRIBUTING.md`, any relevant files in `docs/`) when updating code
- Always update `skills/rw-skill.md` when adding, removing, or modifying any `rw` command or flag (except: `auth`, `api`, `update`, or `config`)
- Always mock network boundaries in tests

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

### Changelog and Releases

- Use [semantic versioning](https://semver.org/) for tagging versions
- Never use a `v` prefix for tagging versions (e.g. use `1.5.2` not `v1.5.2`)
- Only update `CHANGELOG.md` when drafting a release
- Update the compare links when updating CHANGELOG.md
- Call out BREAKING changes with a warning ⚠️ symbol
- Skip `ci` and `build` commits

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
