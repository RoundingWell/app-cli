## 1. Config Model

- [x] 1.1 Add `pub default: Option<BTreeMap<String, String>>` field to the `Profile` struct in `src/config.rs`
- [x] 1.2 Write a unit test confirming an existing profile JSON without a `default` field deserializes without error
- [x] 1.3 Write a unit test confirming a profile JSON with a `default` field deserializes correctly

## 2. Config Default Commands

- [x] 2.1 Add `config default set <key> <value>` subcommand to `src/commands/config.rs`; validate `key` is `team` or `role`, write to active profile's default map, save config
- [x] 2.2 Add `config default get <key>` subcommand; validate key, print value or `null` if unset
- [x] 2.3 Add `config default rm <key>` subcommand; validate key, remove from map if present, no-op if absent, save config
- [x] 2.4 Wire the `config default` subcommand into the CLI argument parser (e.g., `src/main.rs` or wherever `config` subcommands are matched)
- [x] 2.5 Write unit tests for `config default set`: valid keys succeed, unknown key returns error
- [x] 2.6 Write unit tests for `config default get`: returns value when set, prints `null` when unset, unknown key returns error
- [x] 2.7 Write unit tests for `config default rm`: removes key when set, succeeds silently when unset, unknown key returns error

## 3. Clinicians Prepare Integration

- [x] 3.1 In `src/commands/clinicians.rs`, read `default.role` and `default.team` from the active profile before the hard-coded fallback assignments in `prepare`
- [x] 3.2 Apply config default only for non-staff path; staff path (`@roundingwell.com`) remains unchanged
- [x] 3.3 Write a unit test: `prepare` uses config default when set for a non-staff clinician
- [x] 3.4 Write a unit test: `prepare` falls back to `employee`/`NUR` when config default are absent for a non-staff clinician
- [x] 3.5 Write a unit test: `prepare` ignores non-staff config default for a staff clinician

## 4. Docs and Cleanup

- [x] 4.1 Update `README.md` with documentation for `config default set/get/rm` commands
- [x] 4.2 Run `cargo clippy` and fix any warnings
- [x] 4.3 Run `cargo fmt`
- [x] 4.4 Run `cargo test` and confirm all tests pass
