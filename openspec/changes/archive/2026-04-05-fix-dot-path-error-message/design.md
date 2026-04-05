## Context

`build_body` in `src/commands/api.rs` wraps `dot_set` errors with a custom message that tries to identify the conflicting prefix. The original spec mandated this message, but it is misleading for multi-level conflicts (always reports the first segment, not the actual conflicting node). The fix is to keep a custom message but use the full dot-path key and clearer wording.

## Goals / Non-Goals

**Goals:**
- Replace the existing custom error message with `"Unable to set field {f} because it conflicts with another field"`, where `{f}` is the full dot-path key passed to `dot_set`.
- Update tests to assert the new message format.

**Non-Goals:**
- Computing or reporting which existing path caused the conflict.
- Changing any other behavior of `build_body`.

## Decisions

### Use the full key in a plain custom message

Replace the `map_err` closure with one that formats the full key:

```rust
body.dot_set(&k, serde_json::Value::String(v))
    .map_err(|_| anyhow::anyhow!(
        "Unable to set field {} because it conflicts with another field",
        k
    ))?;
```

This is correct for all conflict depths (root-level and multi-level alike) and requires no tree-walking.

**Alternative considered**: Walk the body tree to find the exact conflicting prefix. Rejected—adds complexity with no practical benefit since the full key already tells the user what they tried to set.

**Alternative considered**: Propagate `dot_set`'s error directly (`"Unexpected value reached while traversing path"`). Rejected—too terse and doesn't tell the user which key failed.

## Risks / Trade-offs

- [Risk] Tests asserting the old message string will break. → Expected: update as part of this change.

## Migration Plan

No external API changes. Error message output changes for the conflict case only.

## Open Questions

_None._
