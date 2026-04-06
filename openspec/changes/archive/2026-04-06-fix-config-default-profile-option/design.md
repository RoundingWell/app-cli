## Context

The global `--profile` flag on the `Cli` struct is passed via `cli.profile.as_deref()` to `build_ctx`, which resolves the correct profile for all commands **except** `config`. The `Config` branch in `main.rs` never calls `build_ctx`; it passes raw `&config` / `&mut config` directly to command handlers. The `config default` handlers then call the private helper `active_profile_name(config)` which reads `config.default` (the persistently-configured default profile name) and has no way to receive the CLI override.

## Goals / Non-Goals

**Goals:**
- `rw --profile <name> config default <subcommand>` operates on the named profile, not the configured default.
- Behaviour is unchanged when `--profile` is not supplied.
- All four subcommands (`set`, `get`, `rm`, `list`) are fixed consistently.

**Non-Goals:**
- Changing how other `config` subcommands (`profile`, `updates`) handle `--profile` — they don't need it.
- Changing `build_ctx` or the `AppContext` struct.

## Decisions

### Pass `profile: Option<&str>` into each `default_*` function

**Chosen:** Add a `profile: Option<&str>` parameter to `default_set`, `default_get`, `default_rm`, and `default_list`. Rename (or overload) `active_profile_name` to `resolve_profile_name(config, profile_override)` that prefers the override when present.

**Alternative considered:** Plumb `cli.profile` through `build_ctx` and pass `AppContext` to these functions. Rejected because `AppContext` requires a valid authenticated profile with an organisation and stage; `config default` only needs a profile name and the config map — it doesn't need API credentials.

**Why the chosen approach is minimal:** It is a localised change to four public functions and one private helper. No new types are introduced, and callers in `main.rs` simply pass `cli.profile.as_deref()`.

## Risks / Trade-offs

- [Signature change on four public functions] → All callers are in `main.rs` (one call site each) and in-module tests, which will need updating. The compiler will flag every missed call site, so nothing can slip through.

## Migration Plan

No persistent state changes. No config file format changes. Pure behavioural fix — no migration needed, fully backwards-compatible.

## Open Questions

None.
