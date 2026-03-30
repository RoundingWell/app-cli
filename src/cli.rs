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
    /// Manage basic auth credentials.
    Basic(BasicArgs),
    /// Manage clinicians.
    Clinicians(CliniciansArgs),
    /// Set the default profile.
    Profile(ProfileArgs),
    /// List and manage profiles.
    Profiles(ProfilesArgs),
    /// Update rw to the latest version.
    Update,
}

/// Arguments for the `profile` subcommand.
#[derive(Args, Debug)]
pub struct ProfileArgs {
    /// Profile name to set as default.
    #[arg(value_parser = validate_slug)]
    pub name: String,
}

/// Arguments for the `profiles` subcommand.
#[derive(Args, Debug)]
pub struct ProfilesArgs {
    #[command(subcommand)]
    pub command: Option<ProfilesCommands>,
}

/// Subcommands for `profiles`.
#[derive(Subcommand, Debug)]
pub enum ProfilesCommands {
    /// Add a new profile.
    Add(ProfilesAddArgs),
    /// Remove a profile.
    Rm(ProfilesRmArgs),
}

/// Arguments for `profiles rm`.
#[derive(Args, Debug)]
pub struct ProfilesRmArgs {
    /// Profile name to remove.
    #[arg(value_parser = validate_slug)]
    pub name: String,
}

/// Arguments for `profiles add`.
#[derive(Args, Debug)]
pub struct ProfilesAddArgs {
    /// Profile name to create.
    #[arg(value_parser = validate_slug)]
    pub name: String,

    /// Organization slug.
    #[arg(short = 'o', long, value_parser = validate_slug)]
    pub organization: Option<String>,

    /// Stage.
    #[arg(short = 'g', long)]
    pub stage: Option<Stage>,
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
    /// Assign a role to a clinician by UUID or email.
    Assign(CliniciansAssignArgs),
    /// Enable a clinician by UUID or email.
    Enable(CliniciansTargetArgs),
    /// Disable a clinician by UUID or email.
    Disable(CliniciansTargetArgs),
    /// Prepare a clinician with the appropriate role, team, and workspace memberships.
    Prepare(CliniciansTargetArgs),
}

/// Arguments for `clinicians assign`.
#[derive(Args, Debug)]
pub struct CliniciansAssignArgs {
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

/// Arguments for the `basic` subcommand.
#[derive(Args, Debug)]
pub struct BasicArgs {
    #[command(subcommand)]
    pub command: BasicCommands,
}

/// Subcommands for `basic`.
#[derive(Subcommand, Debug)]
pub enum BasicCommands {
    /// Store basic auth credentials for an organization and stage.
    Set(BasicSetArgs),
}

/// Arguments for `basic set`.
#[derive(Args, Debug)]
pub struct BasicSetArgs {
    /// Username.
    #[arg(short = 'u', long)]
    pub username: Option<String>,

    /// Password (prompted securely if not provided).
    #[arg(short = 'P', long)]
    pub password: Option<String>,
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
