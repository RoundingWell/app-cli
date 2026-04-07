use clap::{Args, Parser, Subcommand, ValueEnum};

/// Validate that a string is a valid slug matching `^[a-z][a-z0-9-]*[a-z0-9]$`.
pub fn validate_slug(s: &str) -> Result<String, String> {
    if s.len() < 2 {
        return Err(format!(
            "'{}' is too short; slugs must be at least 2 characters",
            s
        ));
    }
    let is_valid = is_valid_slug(s);
    if is_valid {
        Ok(s.to_string())
    } else {
        Err(format!(
            "'{}' is not a valid slug; must match ^[a-z][a-z0-9-]*[a-z0-9]$",
            s
        ))
    }
}

fn is_valid_slug(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return false;
    }
    // First char must be lowercase letter
    if !chars[0].is_ascii_lowercase() {
        return false;
    }
    // Last char must be lowercase letter or digit
    let last = *chars.last().unwrap();
    if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
        return false;
    }
    // All chars must be lowercase letter, digit, or hyphen
    for &c in &chars {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return false;
        }
    }
    true
}

/// Stage value for the --stage global option.
#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Prod,
    Sandbox,
    Qa,
    Dev,
    Local,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Prod => write!(f, "prod"),
            Stage::Sandbox => write!(f, "sandbox"),
            Stage::Qa => write!(f, "qa"),
            Stage::Dev => write!(f, "dev"),
            Stage::Local => write!(f, "local"),
        }
    }
}

/// The RoundingWell command line interface.
#[derive(Parser, Debug)]
#[command(name = "rw", about = "RoundingWell CLI", version)]
pub struct Cli {
    /// Profile name.
    #[arg(short = 'p', long, value_parser = validate_slug, global = true)]
    pub profile: Option<String>,

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
    /// Manage clinicians.
    Clinicians(CliniciansArgs),
    /// Manage teams.
    Teams(TeamsArgs),
    /// List roles.
    Roles(RolesArgs),
    /// Update rw to the latest version.
    Update,
    /// Manage CLI configuration, profiles, and update settings.
    Config(ConfigArgs),
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
}

/// Arguments for `teams list`.
#[derive(Args, Debug)]
pub struct TeamsListArgs {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slugs() {
        assert!(validate_slug("demonstration").is_ok());
        assert!(validate_slug("mercy-clinic").is_ok());
        assert!(validate_slug("qa2").is_ok());
        assert!(validate_slug("qa").is_ok());
    }

    #[test]
    fn test_invalid_slugs() {
        assert!(validate_slug("a").is_err()); // too short
        assert!(validate_slug("Mercy-Clinic").is_err()); // uppercase
        assert!(validate_slug("Mercy Clinic").is_err()); // space not allowed
        assert!(validate_slug("-clinic").is_err()); // starts with hyphen
        assert!(validate_slug("mercy-").is_err()); // ends with hyphen
        assert!(validate_slug("mercy_clinic").is_err()); // underscore not allowed
    }
}
