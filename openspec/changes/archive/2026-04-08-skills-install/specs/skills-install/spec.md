## ADDED Requirements

### Requirement: Install agent skill file
The system SHALL provide a `rw skills install` subcommand that writes a Claude Code–compatible skill file to the user's agent skill directory.

#### Scenario: Install to global path
- **WHEN** user runs `rw skills install`
- **THEN** a skill file is written to `~/.claude/skills/rw.md`
- **THEN** a success message is printed indicating the path where the file was written

#### Scenario: Install to project-local path
- **WHEN** user runs `rw skills install --local`
- **THEN** a skill file is written to `.claude/skills/rw.md` relative to the current working directory
- **THEN** a success message is printed indicating the path where the file was written

#### Scenario: Overwrite existing file
- **WHEN** user runs `rw skills install` and a skill file already exists at the target path
- **THEN** the existing file is overwritten with the current skill content
- **THEN** a success message is printed

#### Scenario: No-clobber flag prevents overwrite
- **WHEN** user runs `rw skills install --no-clobber` and a skill file already exists
- **THEN** the command exits without writing
- **THEN** an informational message is printed indicating the file already exists and was not overwritten

#### Scenario: Parent directory is created if missing
- **WHEN** the target directory (e.g. `~/.claude/skills/`) does not exist
- **THEN** the directory is created automatically
- **THEN** the skill file is written successfully

### Requirement: Skill content describes rw usage
The installed skill file SHALL contain accurate, up-to-date documentation for `rw` targeted at Claude Code agents, covering authentication, available commands, flags, and JSON:API conventions.

#### Scenario: Skill file has valid Claude Code frontmatter
- **WHEN** the skill file is written
- **THEN** the file begins with YAML frontmatter containing at least a `name` field and a `description` field

#### Scenario: Skill content covers core command surface
- **WHEN** the skill file is written
- **THEN** the file documents the `auth`, `api`, `clinicians`, `config`, and `update` top-level commands
- **THEN** the file includes usage examples for common workflows
