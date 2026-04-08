## Why

Agents working with `rw` have no structured way to discover how the tool works — they must read source code or docs manually. A `rw skills install` command solves this by delivering an up-to-date, agent-targeted skill file (e.g., for Claude Code) that explains available commands, conventions, and usage patterns directly into the agent's environment.

## What Changes

- Add `rw skills install` subcommand that fetches and installs a Claude Code agent skill
- The installed skill file teaches agents how to use `rw` (commands, flags, JSON:API conventions, auth flow, etc.)
- Add instructions to AGENTS.md requiring developers to keep the skill content in sync whenever tool functionality changes

## Capabilities

### New Capabilities

- `skills-install`: CLI subcommand `rw skills install` that downloads/generates and installs an agent skill for Claude Code users

### Modified Capabilities

<!-- No existing specs have requirement changes -->

## Impact

- New `src/commands/skills.rs` (or similar) module added to the CLI
- Skill file content must stay current with `rw` command surface — AGENTS.md updated to enforce this
- No breaking changes to existing commands or API interaction
