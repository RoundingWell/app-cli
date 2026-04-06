## Context

The `rw config default` subcommand already handles `set`, `get`, and `rm` via a `ConfigDefault` enum in `src/commands/config.rs`. The `default` map is stored in the active profile in `src/config.rs`. Adding `list` is additive and requires no structural changes.

## Goals / Non-Goals

**Goals:**
- Add a `list` variant to the `ConfigDefault` enum
- Print all defined default key/value pairs from the active profile
- Produce no output (and exit 0) when no defaults are set

**Non-Goals:**
- Changing the storage format or allowed keys
- Filtering or sorting output beyond the natural map iteration order

## Decisions

**Plain key=value output format**
Each line prints `<key>=<value>`. This matches common CLI conventions (e.g., `env`) and is easy to parse in scripts. A structured format (JSON, table) would add complexity without clear benefit for this small, fixed-key set.

## Risks / Trade-offs

- None. The `default` map on `Profile` is already a `BTreeMap<String, String>`, so iteration order is alphabetically sorted by key with no extra work.
