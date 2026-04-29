---
name: rw
description: Interact with the RoundingWell API 
triggers:
  # Direction invocation
  - roundingwell
  - /rw
  # Indirect invocation
  - roundingwell actions
  - roundingwell artifacts
  - roundingwell clinicians
  - roundingwell roles
  - roundingwell teams
  - roundingwell workspaces
---

# Using the `rw` CLI

`rw` is the RoundingWell command-line interface. It authenticates against the RoundingWell API and provides commands for managing clinicians, making API requests, and configuring profiles.

## Global Flags

These flags work on every command:

| Flag           | Short | Description                                                            |
|----------------|-------|------------------------------------------------------------------------|
| `--profile`    | `-p`  | Named profile to use                                                   |
| `--auth`       | `-A`  | Use credentials from another profile (overrides only the auth source)  |
| `--config-dir` | `-c`  | Configuration directory path                                           |
| `--json`       |       | Output results as JSON                                                 |

All commands that call the API require a configured profile. Ensure that a profile has been set using `rw config profile show` or pass `--profile` on each invocation.

## Commands

### `rw config` — Configuration & Profiles

**Profile status:**

```sh
rw config profile list  # List all configured profiles
rw config profile show  # Show the active profile
```

### `rw clinicians` — Clinician Management

All targets accept a UUID or email address. Roles accept a UUID or name. Teams accept a UUID or abbreviation.

```sh
# Register a new clinician
rw clinicians register joe@example.com "Joe Smith"
rw clinicians register joe@example.com "Joe Smith" --role employee --team NUR

# Prepare (assigns role, team, visibility, and default workspaces)
rw clinicians prepare joe@example.com

# Enable / disable
rw clinicians enable joe@example.com
rw clinicians disable 60fda0c4-eca0-434a-80d8-fd4e490aa051

# Assign to a team
rw clinicians assign joe@example.com NUR
rw clinicians assign joe@example.com 60c0e3b8-64b6-491f-a502-7346d14b3192

# Grant a role
rw clinicians grant joe@example.com admin
rw clinicians grant joe@example.com 60c0e3b8-64b6-491f-a502-7346d14b3192

# Update a clinician attribute (target can be UUID, email, or "me")
rw clinicians update joe@example.com --field name --value "Jane Doe"
rw clinicians update me --field npi --value 1234567890
rw clinicians update me --field credentials --value "RN,MD"
rw clinicians update me --field npi          # omit --value to clear (sends null)
rw clinicians update me --field credentials  # omit --value to clear (sends [])
```

Fields for `update`: `name`, `email`, `npi`, `credentials`

### `rw actions` — Action Tracing

```sh
# Trace an action's patient, program, and form workspaces (UUID required)
rw actions trace 60fda0c4-eca0-434a-80d8-fd4e490aa051
```

`trace` fetches the action, then its related patient, program, and form. It prints a workspace tree for each and lists any workspaces where the patient and program (via action) or patient and form do not align.

### `rw artifacts` — Artifact Listing

```sh
rw artifacts list <artifact_type> --path <path> --term <term>
```

IMPORTANT: Both `--path` and `--term` are required filters.

### `rw teams` — Team Management

```sh
rw teams list                       # List all teams (id, abbr, name)
rw teams show NUR                   # Show team by abbreviation
rw teams show 60c0e3b8-...          # Show team by UUID
```

### `rw roles` — Role Management

```sh
rw roles list                       # List all roles (id, name, label)
rw roles show admin                 # Show role by name
rw roles show 60c0e3b8-...          # Show role by UUID
```

`show` output includes: `id`, `name`, `label`, `description`, `permissions`.

### `rw workspaces` — Workspace Management

```sh
rw workspaces list                  # List all workspaces (id, slug, name)
rw workspaces show cardiology       # Show workspace by slug
rw workspaces show 60c0e3b8-...     # Show workspace by UUID
```

`show` output includes: `id`, `slug`, `name`, `settings`.

## Common Workflows

### Register and prepare a clinician

```sh
rw clinicians register joe@example.com "Joe Smith" --role employee --team NUR
rw clinicians prepare joe@example.com
```
