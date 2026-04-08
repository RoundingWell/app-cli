## Context

`rw` is a Rust CLI built with Clap. Commands are organized as top-level subcommands in `src/cli.rs` (enum `Commands`) with corresponding handlers under `src/commands/`. There is no `skills` subcommand today.

Claude Code agent skills are Markdown files (with YAML frontmatter) that get written to `~/.claude/skills/` (or a project-local path). When a user runs `rw skills install`, the tool should write a skill file that instructs Claude agents on how to use `rw`.

The skill content must reflect the current command surface of `rw`. AGENTS.md needs a standing instruction to keep the embedded skill content up-to-date whenever commands are added or changed.

## Goals / Non-Goals

**Goals:**
- Add `rw skills install` subcommand
- Write a Claude Code–compatible skill file to `~/.claude/skills/rw.md` (default) or a project-local path (`--local`)
- Embed the skill content directly in the binary (no network fetch required)
- Update AGENTS.md to require skill content updates alongside command changes

**Non-Goals:**
- Supporting agent platforms other than Claude Code (no Cursor, Copilot, etc.)
- Fetching skill content from a remote URL
- Versioning or auto-updating the skill file
- A `skills uninstall` subcommand (out of scope for now)

## Decisions

### Embed skill content in the binary

**Decision:** Use `include_str!` to embed a `skills/rw-skill.md` source file at compile time.

**Rationale:** Keeps installation simple (no network required, works offline, single binary). The skill content is Markdown authored alongside the code, so developers can see and review it in PRs. A network-fetched approach would add latency and a failure mode with no benefit for this use case.

**Alternative considered:** Fetch from a GitHub raw URL at runtime — rejected because it adds a network dependency to a simple local install command.

### Single `install` subcommand (no `list`, `uninstall` yet)

**Decision:** Only implement `rw skills install` for this change.

**Rationale:** Keeps scope tight. Uninstall is trivially done by deleting the file manually; listing installed skills is not yet needed. These can be added later.

### Default install path: `~/.claude/skills/rw.md`

**Decision:** Write to `~/.claude/skills/rw.md` by default; support `--local` flag to write to `.claude/skills/rw.md` in the current working directory.

**Rationale:** The global path installs the skill for all Claude Code sessions. The `--local` flag mirrors the pattern Claude Code itself uses for project-local configuration.

### Overwrite without prompting; add `--no-clobber` for safety

**Decision:** Default behavior is to overwrite an existing file. `--no-clobber` exits without writing if the file already exists.

**Rationale:** Agents running `rw skills install` non-interactively should always get the latest version. Human users who want to preserve manual edits can pass `--no-clobber`.

## Risks / Trade-offs

- **Skill content drift** → Mitigation: AGENTS.md explicitly requires updating `skills/rw-skill.md` when commands are added or changed, and tasks include this as a checklist item.
- **Install path assumptions** → If Claude Code changes its skill directory location, the hardcoded path becomes wrong. Mitigation: path is a single constant, easy to update.
- **Binary size** → Embedding a Markdown file adds negligible size (< 10 KB).
