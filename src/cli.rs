use clap::{Args, Parser, Subcommand};

// `Stage` and `validate_slug` live in `crate::domain`. Re-exported here so
// existing call sites that imported them via `crate::cli::*` keep working.
pub use crate::domain::{validate_slug, Stage};

/// The RoundingWell command line interface.
#[derive(Parser, Debug)]
#[command(name = "rw", about = "RoundingWell CLI", version)]
pub struct Cli {
    /// Profile name.
    #[arg(short = 'p', long, value_parser = validate_slug, global = true)]
    pub profile: Option<String>,

    /// Profile whose stored credentials should be used for this invocation.
    #[arg(short = 'A', long, value_parser = validate_slug, global = true)]
    pub auth: Option<String>,

    /// Output results as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Config directory path (must already exist).
    #[arg(short = 'c', long, global = true)]
    pub config_dir: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Authenticate with RoundingWell.
    Auth(AuthArgs),
    /// Make an API request.
    Api(ApiArgs),
    /// List artifacts.
    Artifacts(ArtifactsArgs),
    /// Manage clinicians.
    Clinicians(CliniciansArgs),
    /// Manage teams.
    Teams(TeamsArgs),
    /// List roles.
    Roles(RolesArgs),
    /// List workspaces.
    Workspaces(WorkspacesArgs),
    /// Update rw to the latest version.
    Update,
    /// Manage CLI configuration, profiles, and update settings.
    Config(ConfigArgs),
    /// Manage agent skills.
    Skills(SkillsArgs),
}

/// Arguments for the `artifacts` subcommand.
#[derive(Args, Debug)]
pub struct ArtifactsArgs {
    #[command(subcommand)]
    pub command: ArtifactsCommands,
}

/// Subcommands for `artifacts`.
#[derive(Subcommand, Debug)]
pub enum ArtifactsCommands {
    /// List artifacts filtered by type, path, and term.
    List(ArtifactsListArgs),
}

/// Arguments for `artifacts list`.
#[derive(Args, Debug)]
pub struct ArtifactsListArgs {
    /// Artifact type to filter by.
    pub artifact_type: String,
    /// Path filter.
    #[arg(long, required = true)]
    pub path: String,
    /// Search term filter.
    #[arg(long, required = true)]
    pub term: String,
}

/// Arguments for the `workspaces` subcommand.
#[derive(Args, Debug)]
pub struct WorkspacesArgs {
    #[command(subcommand)]
    pub command: WorkspacesCommands,
}

/// Subcommands for `workspaces`.
#[derive(Subcommand, Debug)]
pub enum WorkspacesCommands {
    /// List all workspaces.
    List(WorkspacesListArgs),
    /// Show a workspace by UUID or slug.
    Show(WorkspacesShowArgs),
}

/// Arguments for `workspaces list`.
#[derive(Args, Debug)]
pub struct WorkspacesListArgs {}

/// Arguments for `workspaces show`.
#[derive(Args, Debug)]
pub struct WorkspacesShowArgs {
    /// Workspace UUID or slug.
    pub target: String,
}

/// Arguments for the `roles` subcommand.
#[derive(Args, Debug)]
pub struct RolesArgs {
    #[command(subcommand)]
    pub command: RolesCommands,
}

/// Subcommands for `roles`.
#[derive(Subcommand, Debug)]
pub enum RolesCommands {
    /// List all roles.
    List(RolesListArgs),
    /// Show a role by UUID or name.
    Show(RolesShowArgs),
}

/// Arguments for `roles list`.
#[derive(Args, Debug)]
pub struct RolesListArgs {}

/// Arguments for `roles show`.
#[derive(Args, Debug)]
pub struct RolesShowArgs {
    /// Role UUID or name.
    pub target: String,
}

/// Arguments for the `teams` subcommand.
#[derive(Args, Debug)]
pub struct TeamsArgs {
    #[command(subcommand)]
    pub command: TeamsCommands,
}

/// Subcommands for `teams`.
#[derive(Subcommand, Debug)]
pub enum TeamsCommands {
    /// List all teams.
    List(TeamsListArgs),
    /// Show a team by UUID or abbreviation.
    Show(TeamsShowArgs),
}

/// Arguments for `teams list`.
#[derive(Args, Debug)]
pub struct TeamsListArgs {}

/// Arguments for `teams show`.
#[derive(Args, Debug)]
pub struct TeamsShowArgs {
    /// Team UUID or abbreviation.
    pub target: String,
}

/// Arguments for the `config` subcommand.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Subcommands for `config`.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Manage profiles.
    Profile(ConfigProfileArgs),
    /// Manage automatic update settings.
    Updates(ConfigUpdatesArgs),
    /// Manage per-profile default values.
    Default(ConfigDefaultArgs),
}

/// Arguments for `config default`.
#[derive(Args, Debug)]
pub struct ConfigDefaultArgs {
    #[command(subcommand)]
    pub command: ConfigDefaultCommands,
}

/// Subcommands for `config default`.
#[derive(Subcommand, Debug)]
pub enum ConfigDefaultCommands {
    /// Set a default value for the active profile.
    Set(ConfigDefaultSetArgs),
    /// Get a default value for the active profile.
    Get(ConfigDefaultGetArgs),
    /// Remove a default value from the active profile.
    Rm(ConfigDefaultRmArgs),
    /// List all defined default values for the active profile.
    List,
}

/// Arguments for `config default set`.
#[derive(Args, Debug)]
pub struct ConfigDefaultSetArgs {
    /// Key to set (team, role).
    pub key: String,
    /// Value to assign.
    pub value: String,
}

/// Arguments for `config default get`.
#[derive(Args, Debug)]
pub struct ConfigDefaultGetArgs {
    /// Key to read (team, role).
    pub key: String,
}

/// Arguments for `config default rm`.
#[derive(Args, Debug)]
pub struct ConfigDefaultRmArgs {
    /// Key to remove (team, role).
    pub key: String,
}

/// Arguments for `config profile`.
#[derive(Args, Debug)]
pub struct ConfigProfileArgs {
    #[command(subcommand)]
    pub command: ConfigProfileCommands,
}

/// Subcommands for `config profile`.
#[derive(Subcommand, Debug)]
pub enum ConfigProfileCommands {
    /// List all profiles.
    List,
    /// Show the active profile.
    Show,
    /// Set a profile as the default.
    Use(ConfigProfileUseArgs),
    /// Update fields on an existing profile.
    Set(ConfigProfileSetArgs),
    /// Remove a profile.
    Rm(ConfigProfileRmArgs),
    /// Add a new profile.
    Add(ConfigProfileAddArgs),
    /// Save basic auth credentials for a profile.
    Auth(ConfigProfileAuthArgs),
}

/// Arguments for `config profile use`.
#[derive(Args, Debug)]
pub struct ConfigProfileUseArgs {
    /// Profile name.
    #[arg(value_parser = validate_slug)]
    pub name: String,
}

/// Arguments for `config profile set`.
#[derive(Args, Debug)]
pub struct ConfigProfileSetArgs {
    /// Profile name.
    #[arg(value_parser = validate_slug)]
    pub name: String,
    /// Organization slug.
    #[arg(short = 'o', long, value_parser = validate_slug)]
    pub organization: Option<String>,
    /// Stage.
    #[arg(short = 'g', long)]
    pub stage: Option<Stage>,
}

/// Arguments for `config profile rm`.
#[derive(Args, Debug)]
pub struct ConfigProfileRmArgs {
    /// Profile name.
    #[arg(value_parser = validate_slug)]
    pub name: String,
    /// Skip confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

/// Arguments for `config profile add`.
#[derive(Args, Debug)]
pub struct ConfigProfileAddArgs {
    /// Profile name.
    #[arg(value_parser = validate_slug)]
    pub name: String,
    /// Organization slug.
    #[arg(short = 'o', long, value_parser = validate_slug)]
    pub organization: Option<String>,
    /// Stage.
    #[arg(short = 'g', long)]
    pub stage: Option<Stage>,
    /// Set as default profile after creation.
    #[arg(long = "use")]
    pub make_active: bool,
}

/// Arguments for `config profile auth`.
#[derive(Args, Debug)]
pub struct ConfigProfileAuthArgs {
    /// Profile name.
    #[arg(value_parser = validate_slug)]
    pub name: String,
    /// Username.
    #[arg(short = 'u', long)]
    pub username: Option<String>,
    /// Password (prompted securely if not provided).
    #[arg(short = 'P', long)]
    pub password: Option<String>,
}

/// Arguments for `config updates`.
#[derive(Args, Debug)]
pub struct ConfigUpdatesArgs {
    #[command(subcommand)]
    pub command: ConfigUpdatesCommands,
}

/// Subcommands for `config updates`.
#[derive(Subcommand, Debug)]
pub enum ConfigUpdatesCommands {
    /// Show the current update setting.
    Show,
    /// Enable automatic updates.
    Enable,
    /// Disable automatic updates.
    Disable,
}

/// Arguments for the `clinician` subcommand.
#[derive(Args, Debug)]
pub struct CliniciansArgs {
    #[command(subcommand)]
    pub command: CliniciansCommands,
}

/// Subcommands for `clinician`.
#[derive(Subcommand, Debug)]
pub enum CliniciansCommands {
    /// Assign a clinician to a team by UUID or email.
    Assign(CliniciansAssignArgs),
    /// Grant a role to a clinician by UUID or email.
    Grant(CliniciansGrantArgs),
    /// Enable a clinician by UUID or email.
    Enable(CliniciansTargetArgs),
    /// Disable a clinician by UUID or email.
    Disable(CliniciansTargetArgs),
    /// Prepare a clinician with the appropriate role, team, and workspace memberships.
    Prepare(CliniciansTargetArgs),
    /// Register a new clinician.
    Register(CliniciansRegisterArgs),
    /// Show a clinician by UUID, email, or "me".
    Show(CliniciansTargetArgs),
    /// Update a clinician attribute by UUID, email, or "me".
    Update(CliniciansUpdateArgs),
}

/// Arguments for `clinicians assign`.
#[derive(Args, Debug)]
pub struct CliniciansAssignArgs {
    /// Clinician UUID or email address.
    pub target: String,
    /// Team UUID or abbreviation.
    pub team: String,
}

/// Arguments for `clinicians grant`.
#[derive(Args, Debug)]
pub struct CliniciansGrantArgs {
    /// Clinician UUID or email address.
    pub target: String,
    /// Role UUID or name.
    pub role: String,
}

/// Arguments for `clinician enable` / `clinician disable`.
#[derive(Args, Debug)]
pub struct CliniciansTargetArgs {
    /// Clinician UUID or email address.
    pub target: String,
}

/// Arguments for `clinicians update`.
#[derive(Args, Debug)]
pub struct CliniciansUpdateArgs {
    /// Clinician UUID, email address, or "me".
    pub target: String,
    /// Field to update (name, email, npi, credentials).
    #[arg(long)]
    pub field: String,
    /// New value for the field (omit to clear npi or credentials).
    #[arg(long)]
    pub value: Option<String>,
}

/// Arguments for `clinicians register`.
#[derive(Args, Debug)]
pub struct CliniciansRegisterArgs {
    /// Email address for the new clinician.
    pub email: String,
    /// Full name of the new clinician.
    pub name: String,
    /// Role UUID or name.
    #[arg(long)]
    pub role: Option<String>,
    /// Team UUID or abbreviation.
    #[arg(long)]
    pub team: Option<String>,
}

/// Arguments for the `auth` subcommand.
#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

/// Subcommands for `auth`.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Log in to RoundingWell.
    Login,
    /// Show current authentication status.
    Status,
    /// Print the Authorization header value for the current profile.
    Header,
    /// Log out and remove stored credentials.
    Logout,
}

/// Arguments for the `api` subcommand.
#[derive(Args, Debug)]
pub struct ApiArgs {
    /// API endpoint path (e.g., "clinicians").
    pub endpoint: String,

    /// HTTP method to use.
    #[arg(short = 'X', long = "method", default_value = "GET")]
    pub method: String,

    /// Additional HTTP headers (e.g., "Accept: application/json").
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Request body fields as key=value pairs (implies POST if no method set).
    #[arg(short = 'f', long = "field")]
    pub fields: Vec<String>,

    /// Filter output with a jq expression.
    #[arg(short = 'q', long)]
    pub jq: Option<String>,

    /// Output raw JSON without pretty-printing.
    #[arg(long)]
    pub raw: bool,
}

/// Arguments for the `skills` subcommand.
#[derive(Args, Debug)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommands,
}

/// Subcommands for `skills`.
#[derive(Subcommand, Debug)]
pub enum SkillsCommands {
    /// Install the rw agent skill for Claude Code.
    Install(SkillsInstallArgs),
}

/// Arguments for `skills install`.
#[derive(Args, Debug)]
pub struct SkillsInstallArgs {
    /// Install to `.claude/` instead of global `~/.claude/`.
    #[arg(long)]
    pub local: bool,

    /// Do not overwrite an existing skill file.
    #[arg(long)]
    pub no_clobber: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_flag_long() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rw", "--auth", "mercy", "auth", "status"]).unwrap();
        assert_eq!(cli.auth.as_deref(), Some("mercy"));
    }

    #[test]
    fn test_auth_flag_short() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rw", "-A", "mercy", "auth", "status"]).unwrap();
        assert_eq!(cli.auth.as_deref(), Some("mercy"));
    }

    #[test]
    fn test_auth_flag_default_none() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rw", "auth", "status"]).unwrap();
        assert!(cli.auth.is_none());
    }

    #[test]
    fn test_auth_flag_validates_slug() {
        use clap::Parser;
        // Uppercase letter should fail slug validation.
        let err = Cli::try_parse_from(["rw", "-A", "Mercy", "auth", "status"]).unwrap_err();
        assert!(err.to_string().contains("not a valid slug"));
    }

    #[test]
    fn test_auth_flag_propagates_to_subcommands() {
        use clap::Parser;
        // The flag is global, so it works after the subcommand too.
        let cli = Cli::try_parse_from(["rw", "auth", "status", "-A", "mercy"]).unwrap();
        assert_eq!(cli.auth.as_deref(), Some("mercy"));
    }
}
