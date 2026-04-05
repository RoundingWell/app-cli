## Context

The CLI currently exposes `clinicians assign <role>` to grant a role to a clinician. The subcommand name "assign" is being replaced with "grant" to better match RBAC terminology. This is a straightforward rename — no behavior changes, no API changes.

## Goals / Non-Goals

**Goals:**
- Rename the `assign` subcommand to `grant` in the CLI definition
- Rename the internal `assign` function to `grant`
- Update all tests, output messages, and docs to use `grant`

**Non-Goals:**
- Changing the behavior of the command
- Modifying the underlying API calls
- Providing a deprecation alias for `assign`

## Decisions

**Remove `assign` without a deprecation alias**
The CLI is a developer-facing tool distributed via `cargo install`. A hard rename (no alias) keeps the interface clean. Users who rely on `assign` will get a clear "unknown subcommand" error that prompts them to check the docs.

## Risks / Trade-offs

- [Breaking change for existing scripts using `clinicians assign`] → Documented as **BREAKING** in the changelog; users must update their scripts.
