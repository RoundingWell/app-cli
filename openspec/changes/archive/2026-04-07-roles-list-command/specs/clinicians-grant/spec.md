## MODIFIED Requirements

### Requirement: Grant a role to a clinician
The CLI SHALL provide a `clinicians grant <role>` subcommand that grants a role to a clinician, replacing the former `clinicians assign <role>` subcommand. Role resolution SHALL use types from the `roles` module.

#### Scenario: Grant role by clinician email and role name
- **WHEN** the user runs `rw clinicians grant <email> <role-name>`
- **THEN** the system finds the clinician by email, finds the role by name using the shared `roles` module types, and assigns the role via the API, confirming success with a plain-text message

#### Scenario: Grant role by clinician UUID and role UUID
- **WHEN** the user runs `rw clinicians grant <clinician-uuid> <role-uuid>`
- **THEN** the system uses the UUIDs directly and assigns the role via the API, confirming success with a plain-text message

#### Scenario: Role not found returns an error
- **WHEN** the user runs `rw clinicians grant <target> <role>` and the role does not exist
- **THEN** the system returns an error indicating the role was not found
