## 1. Tests (TDD — write failing tests first)

- [x] 1.1 Add test for `default_set` with an explicit profile override (non-default profile)
- [x] 1.2 Add test for `default_get` with an explicit profile override
- [x] 1.3 Add test for `default_rm` with an explicit profile override
- [x] 1.4 Add test for `default_list` with an explicit profile override
- [x] 1.5 Verify tests fail (they reference the new signature that doesn't exist yet)

## 2. Core Implementation

- [x] 2.1 Rename `active_profile_name(config)` to `resolve_profile_name(config, profile_override: Option<&str>)` — prefer override when `Some`, fall back to `config.default` otherwise
- [x] 2.2 Add `profile: Option<&str>` parameter to `default_set` and update its body to call `resolve_profile_name`
- [x] 2.3 Add `profile: Option<&str>` parameter to `default_get` and update its body to call `resolve_profile_name`
- [x] 2.4 Add `profile: Option<&str>` parameter to `default_rm` and update its body to call `resolve_profile_name`
- [x] 2.5 Add `profile: Option<&str>` parameter to `default_list` and update its body to call `resolve_profile_name`

## 3. Wire Up in main.rs

- [x] 3.1 Pass `cli.profile.as_deref()` to `default_set` in the `ConfigDefaultCommands::Set` dispatch arm
- [x] 3.2 Pass `cli.profile.as_deref()` to `default_get` in the `ConfigDefaultCommands::Get` dispatch arm
- [x] 3.3 Pass `cli.profile.as_deref()` to `default_rm` in the `ConfigDefaultCommands::Rm` dispatch arm
- [x] 3.4 Pass `cli.profile.as_deref()` to `default_list` in the `ConfigDefaultCommands::List` dispatch arm

## 4. Quality

- [x] 4.1 Run `cargo test` — all tests pass
- [x] 4.2 Run `cargo clippy` — no warnings
- [x] 4.3 Run `cargo fmt` — no formatting changes
