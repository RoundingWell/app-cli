## Context

The `roles` module currently only supports `roles list`. The `resolve_role` helper—which resolves a UUID-or-name string to a `(id, name)` tuple by calling `GET /roles`—lives in `clinicians.rs` but logically belongs in `roles.rs`. The `show` command displays full role details (id, name, label, description, permissions) resolved entirely from the `GET /roles` list response. There is no single-resource endpoint.

## Goals / Non-Goals

**Goals:**
- Add `roles show <target>` that displays id, name, label, description, and permissions
- Move `resolve_role` from `clinicians.rs` to `roles.rs` and re-export it for clinicians use
- Follow the existing patterns: JSON:API deserialization, `CommandOutput` trait, `--json` flag

**Non-Goals:**
- Modifying the permissions model or what permissions are returned
- Pagination of roles or permissions
- Caching role lookups

## Decisions

**Resolve everything from `GET /roles`**
There is no single-resource endpoint. All role fields (id, name, label, description, permissions) are returned by the list endpoint. `roles show` calls `GET /roles` once, finds the matching resource by UUID or name, and renders it. This means `resolve_role` can be extended (or a separate helper added) to return the full `RoleResource` instead of just `(id, name)` when needed.

**Move `resolve_role` to `roles.rs`, keep `pub(crate)`**
`resolve_role` calls `GET /roles` and uses `RoleListResponse`, which is defined in `roles.rs`. Keeping it in clinicians required a cross-module type reference (`super::roles::RoleListResponse`). Moving it to `roles.rs` eliminates that coupling. Clinicians imports it as `super::roles::resolve_role`.

**Display permissions as a bulleted list (plain) / array (JSON)**
Permissions are a list of strings. In plain output, render them below the role details as a list. In JSON output, include them in the object. This matches how similar detail views work in CLIs.

## Risks / Trade-offs

- [Risk] `resolve_role` signature change could break clinicians callers → Mitigation: keep the existing `resolve_role` signature identical; only move the definition
