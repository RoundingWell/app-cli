## MODIFIED Requirements

### Requirement: Prepare a clinician for onboarding
The `clinicians prepare` subcommand SHALL assign a role, team, and workspace access to an existing clinician. For staff clinicians (identified by a `@roundingwell.com` email domain), the command SHALL always use `role="rw"` and `team="OT"`. For non-staff clinicians, the command SHALL use the active profile's `default.role` and `default.team` config values, falling back to `role="employee"` and `team="NUR"` when those values are not set.

#### Scenario: Prepare non-staff clinician with no config default set
- **WHEN** neither `default.role` nor `default.team` is set in the active profile and the clinician email does not end with `@roundingwell.com`
- **THEN** the CLI SHALL use `role="employee"` and `team="NUR"` and proceed with the prepare workflow

#### Scenario: Prepare non-staff clinician with config default set
- **WHEN** `default.role="physician"` and `default.team="ICU"` are set in the active profile and the clinician email does not end with `@roundingwell.com`
- **THEN** the CLI SHALL use `role="physician"` and `team="ICU"` instead of the hard-coded fallbacks and proceed with the prepare workflow

#### Scenario: Prepare staff clinician ignores non-staff default
- **WHEN** the clinician email ends with `@roundingwell.com`
- **THEN** the CLI SHALL use `role="rw"` and `team="OT"` regardless of any config default and proceed with the prepare workflow
