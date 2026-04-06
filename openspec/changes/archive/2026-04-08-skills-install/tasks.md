## 1. Skill Content File

- [x] 1.1 Create `skills/rw-skill.md` with YAML frontmatter (`name`, `description`) and documentation of some `rw` commands (`auth`, `clinicians`, `config`) and all related flags, targeted at Claude Code agents
- [x] 1.2 Include usage examples for common workflows (profile show, clinician operations)

## 2. CLI Wiring

- [x] 2.1 Add `SkillsArgs` and `SkillsCommands` (with `Install` variant) to `src/cli.rs`
- [x] 2.2 Add `Skills(SkillsArgs)` variant to the top-level `Commands` enum in `src/cli.rs`
- [x] 2.3 Add `--local` flag and `--no-clobber` flag to the `Install` subcommand args

## 3. Command Implementation

- [x] 3.1 Create `src/commands/skills.rs` with `run_install` function
- [x] 3.2 Use `include_str!("../../skills/rw-skill.md")` to embed skill content at compile time
- [x] 3.3 Resolve target path: `~/.claude/skills/rw.md` (global) or `.claude/skills/rw.md` (local)
- [x] 3.4 Create parent directories if they don't exist (`fs::create_dir_all`)
- [x] 3.5 Respect `--no-clobber`: if file exists and flag is set, print info message and exit without writing
- [x] 3.6 Write the skill file and print success message with the resolved path
- [x] 3.7 Register `skills.rs` in `src/commands/mod.rs` and dispatch from `src/main.rs`

## 4. Tests

- [x] 4.1 Write unit tests for `run_install`: global path, local path, overwrite, no-clobber, missing parent directory

## 5. Documentation & Maintenance

- [x] 5.1 Update `README.md` to document `rw skills install` with `--local` and `--no-clobber` flags
- [x] 5.2 Update `AGENTS.md` to add an instruction requiring `skills/rw-skill.md` to be kept in sync whenever commands are added or changed
- [x] 5.3 Run `cargo clippy`, `cargo fmt`, and `cargo test`
