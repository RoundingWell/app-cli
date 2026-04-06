## 1. Tests

- [x] 1.1 Add a failing test for `config default list` with multiple defaults set
- [x] 1.2 Add a failing test for `config default list` with no defaults set
- [x] 1.3 Add a failing test for `config default list` with only one default set

## 2. Implementation

- [x] 2.1 Add `List` variant to the `ConfigDefault` enum in `src/commands/config.rs`
- [x] 2.2 Implement the `List` handler: iterate the `default` map and print `key=value` lines
- [x] 2.3 Register `list` as a subcommand in the CLI arg definition

## 3. Docs & Quality

- [x] 3.1 Update `README.md` to document the `config default list` subcommand
- [x] 3.2 Run `cargo clippy` and `cargo fmt` and fix any issues
- [x] 3.3 Run `cargo test` and confirm all tests pass
