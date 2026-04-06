## Context

The `clinicians prepare` command hard-codes `role="employee"` and `team="NUR"` for non-staff clinicians (and `role="rw"`, `team="OT"` for staff). These values are environment-specific and should be user-configurable. The config system already supports per-profile settings stored in `~/.config/rw/config.json` as a JSON map of `Profile` structs. The `Profile` struct currently has `organization` and `stage` fields only.

## Goals / Non-Goals

**Goals:**
- Add a `defaults` map to the `Profile` struct to store arbitrary key/value string pairs, restricted to `team` and `role` at the command layer
- Expose `config default set/get/rm` subcommands to manage these values
- Update `clinicians prepare` to read `defaults.role` and `defaults.team` from the active profile, falling back to existing hard-coded values when unset

**Non-Goals:**
- Supporting arbitrary keys beyond `team` and `role`
- Changing the config file format or file location
- Affecting any other command that uses role/team (e.g., `clinicians register` explicit flags are unchanged)

## Decisions

### Store defaults as `BTreeMap<String, String>` in `Profile`

**Decision:** Add `pub default: Option<BTreeMap<String, String>>` to the `Profile` struct.

**Rationale:** Using an open map at the storage layer (rather than typed fields) keeps the config schema flexible and avoids a breaking migration if more default keys are added later. Restriction to `team`/`role` is enforced at the command layer, not the storage layer. `Option` preserves backwards compatibility — existing profiles without a `default` key deserialize cleanly.

**Alternative considered:** Named fields (`pub default_role: Option<String>`, `pub default_team: Option<String>`). Rejected because it requires a schema migration each time a new default key is added and adds more boilerplate for a simple key/value store.

### Validate allowed keys in command handlers, not in `Config`

**Decision:** The `config default set/get/rm` handlers accept a `key: &str` parameter and validate it against an allowlist `["team", "role"]` before touching the config. The `Config`/`Profile` structs remain key-agnostic.

**Rationale:** Keeps storage generic while still giving users a clear error at the CLI layer if they try to set an unsupported key. Testing the allowlist at the command layer is straightforward.

### Fall back to hard-coded values in `clinicians prepare`

**Decision:** After reading `defaults.role` and `defaults.team` from the active profile, fall back to `"employee"` / `"NUR"` (non-staff) and `"rw"` / `"OT"` (staff) when the config values are absent.

**Rationale:** Preserves existing behavior for users who have not configured defaults, requiring no migration or documentation of breaking changes.

## Risks / Trade-offs

- [Risk] Config file written by `config default set` could be corrupted if the process is interrupted mid-write → Mitigation: The existing `Config::save()` pattern (write then persist) is reused; no additional risk introduced.
- [Risk] Using a generic map means invalid key names could slip into the config file via direct file edits → Mitigation: The CLI validates on read for `clinicians prepare` by simply ignoring unknown keys; users manually editing the file are out of scope.
- [Trade-off] `config default get` produces no output and exits zero for undefined keys, following POSIX conventions. Scripts that need to distinguish "set but empty" from "not set" can use the exit code of `config default rm` as a probe, but this scenario is considered out of scope.
