use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use rw::api::resolve_api;
use rw::cli::{self, Cli, Commands};
use rw::commands;
use rw::config::{self, config_path, default_config_dir, load_config, resolve_profile, AppContext};
use rw::migration;
use rw::output::Output;
use rw::version_check;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let out = Output { json: cli.json };

    if let Err(e) = run(cli, &out).await {
        out.error(&e);
        std::process::exit(1);
    }
}

fn build_ctx(
    config: &config::Config,
    profile: Option<&str>,
    auth: Option<&str>,
    stage_override: Option<&cli::Stage>,
    config_dir: PathBuf,
) -> Result<AppContext> {
    let (profile, organization, stage) = resolve_profile(config, profile, stage_override)?;
    let auth_profile = config::resolve_auth_profile(config, &profile, auth)?;
    let auth_stage = config
        .profiles
        .get(&auth_profile)
        .map(|p| p.stage.clone())
        .unwrap_or_else(|| stage.clone());
    let base_url = resolve_api(&organization, &stage);
    let defaults = config
        .profiles
        .get(&profile)
        .and_then(|p| p.default.clone())
        .unwrap_or_default();
    Ok(AppContext {
        config_dir,
        profile,
        auth_profile,
        stage,
        auth_stage,
        base_url,
        defaults,
    })
}

/// Returns an error when `--auth` is set but the command is `auth login` or
/// `auth logout`, where overriding the credential source is contradictory
/// (login *creates* credentials, logout *removes* them — both for a specific
/// profile, not borrowed from elsewhere).
fn check_auth_compatible(cmd: &Commands, auth: Option<&str>) -> Result<()> {
    if auth.is_none() {
        return Ok(());
    }
    if let Commands::Auth(args) = cmd {
        match args.command {
            cli::AuthCommands::Login => {
                anyhow::bail!("--auth cannot be used with `rw auth login`");
            }
            cli::AuthCommands::Logout => {
                anyhow::bail!("--auth cannot be used with `rw auth logout`");
            }
            cli::AuthCommands::Status | cli::AuthCommands::Header => {}
        }
    }
    Ok(())
}

/// Returns an error when `--stage` is set but the command is `auth login` or
/// `auth logout`. `login` mints its token from the overridden stage's WorkOS
/// tenant yet stores it under the profile, whose refreshes use the profile's
/// *configured* stage — across tenants that leaves a token the profile can never
/// refresh. Within a tenant the minted token is identical either way, so the
/// override buys nothing. `logout` merely deletes the profile's credential file,
/// so a stage override is meaningless there.
fn check_stage_compatible(cmd: &Commands, stage: Option<&cli::Stage>) -> Result<()> {
    if stage.is_none() {
        return Ok(());
    }
    if let Commands::Auth(args) = cmd {
        match args.command {
            cli::AuthCommands::Login => {
                anyhow::bail!("--stage cannot be used with `rw auth login`");
            }
            cli::AuthCommands::Logout => {
                anyhow::bail!("--stage cannot be used with `rw auth logout`");
            }
            cli::AuthCommands::Status | cli::AuthCommands::Header => {}
        }
    }
    Ok(())
}

async fn run(cli: Cli, out: &Output) -> Result<()> {
    let config_dir: PathBuf = if let Some(ref dir) = cli.config_dir {
        let path = PathBuf::from(dir);
        if !path.is_dir() {
            anyhow::bail!("config directory does not exist: {}", path.display());
        }
        path
    } else {
        default_config_dir()?
    };

    let cfg_path = config_path(&config_dir);
    let mut config = load_config(&cfg_path)?;
    migration::run_migrations(&config_dir, &mut config)?;

    // Record the current binary version so the next run can detect upgrades and
    // apply any new migrations.  This must happen *after* run_migrations so that
    // migration checks compare against the previously-installed version, not the
    // one that is about to start running.
    let current_version = env!("CARGO_PKG_VERSION");
    if config.version.as_deref() != Some(current_version) {
        config.version = Some(current_version.to_string());
        config::save_config_to(&config, &cfg_path)?;
    }

    // Run version check and auto-update before the command, except when the
    // user is explicitly running `rw update` (to avoid a redundant double-check),
    // `rw config` (they may be disabling auto updates), or `rw skills` (purely
    // local file-write with no API interaction).
    if !matches!(
        cli.command,
        Commands::Update | Commands::Config(_) | Commands::Skills(_)
    ) {
        version_check::check_and_update(&config_dir, &mut config, &cfg_path, out).await;
    }

    check_auth_compatible(&cli.command, cli.auth.as_deref())?;
    check_stage_compatible(&cli.command, cli.stage.as_ref())?;

    let profile_override = cli.profile.clone();
    let auth_override = cli.auth.clone();
    let stage_override = cli.stage.clone();

    match cli.command {
        Commands::Actions(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::actions::dispatch(args, &ctx, out).await?;
        }
        Commands::Artifacts(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::artifacts::dispatch(args, &ctx, out).await?;
        }
        Commands::Auth(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::auth::dispatch(args, &ctx, out).await?;
        }
        Commands::Clinicians(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::clinicians::dispatch(args, &ctx, out).await?;
        }
        Commands::Teams(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::teams::dispatch(args, &ctx, out).await?;
        }
        Commands::Roles(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::roles::dispatch(args, &ctx, out).await?;
        }
        Commands::Workspaces(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::workspaces::dispatch(args, &ctx, out).await?;
        }
        Commands::Api(args) => {
            let ctx = build_ctx(
                &config,
                profile_override.as_deref(),
                auth_override.as_deref(),
                stage_override.as_ref(),
                config_dir,
            )?;
            commands::api::dispatch(args, &ctx, out).await?;
        }
        Commands::Update => commands::update::dispatch(out).await?,
        Commands::Config(args) => {
            commands::config::dispatch(
                args,
                &mut config,
                &cfg_path,
                &config_dir,
                profile_override.as_deref(),
                stage_override.as_ref(),
                out,
            )
            .await?
        }
        Commands::Skills(args) => commands::skills::dispatch(args, out)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli::{AuthArgs, AuthCommands};

    fn login_cmd() -> Commands {
        Commands::Auth(AuthArgs {
            command: AuthCommands::Login,
        })
    }

    fn logout_cmd() -> Commands {
        Commands::Auth(AuthArgs {
            command: AuthCommands::Logout,
        })
    }

    fn status_cmd() -> Commands {
        Commands::Auth(AuthArgs {
            command: AuthCommands::Status,
        })
    }

    fn header_cmd() -> Commands {
        Commands::Auth(AuthArgs {
            command: AuthCommands::Header,
        })
    }

    #[test]
    fn test_check_auth_compatible_allows_login_without_override() {
        assert!(check_auth_compatible(&login_cmd(), None).is_ok());
    }

    #[test]
    fn test_check_auth_compatible_rejects_login_with_override() {
        let err = check_auth_compatible(&login_cmd(), Some("mercy")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--auth"));
        assert!(msg.contains("auth login"));
    }

    #[test]
    fn test_check_auth_compatible_rejects_logout_with_override() {
        let err = check_auth_compatible(&logout_cmd(), Some("mercy")).unwrap_err();
        assert!(err.to_string().contains("auth logout"));
    }

    #[test]
    fn test_check_auth_compatible_allows_status_with_override() {
        assert!(check_auth_compatible(&status_cmd(), Some("mercy")).is_ok());
    }

    #[test]
    fn test_check_auth_compatible_allows_other_commands_with_override() {
        let cmd = Commands::Update;
        assert!(check_auth_compatible(&cmd, Some("mercy")).is_ok());
    }

    #[test]
    fn test_build_ctx_auth_stage_matches_active_stage_without_override() {
        use cli::Stage;
        use config::{Config, Profile};

        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Dev,
                default: None,
            },
        );

        let ctx = build_ctx(&config, Some("demo"), None, None, PathBuf::from("/tmp")).unwrap();
        assert_eq!(ctx.stage, Stage::Dev);
        assert_eq!(ctx.auth_stage, Stage::Dev);
    }

    #[test]
    fn test_build_ctx_auth_stage_uses_override_profile_stage() {
        use cli::Stage;
        use config::{Config, Profile};

        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Dev,
                default: None,
            },
        );
        config.profiles.insert(
            "service".to_string(),
            Profile {
                organization: "service".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );

        let ctx = build_ctx(
            &config,
            Some("demo"),
            Some("service"),
            None,
            PathBuf::from("/tmp"),
        )
        .unwrap();
        assert_eq!(ctx.stage, Stage::Dev);
        assert_eq!(ctx.auth_stage, Stage::Prod);
    }

    #[test]
    fn test_build_ctx_stage_override_replaces_stage_and_base_url() {
        use cli::Stage;
        use config::{Config, Profile};

        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );

        let ctx = build_ctx(
            &config,
            Some("demo"),
            None,
            Some(&Stage::Local),
            PathBuf::from("/tmp"),
        )
        .unwrap();
        assert_eq!(ctx.stage, Stage::Local);
        assert_eq!(ctx.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_build_ctx_stage_override_leaves_auth_stage_at_profile_stage() {
        use cli::Stage;
        use config::{Config, Profile};

        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );

        let ctx = build_ctx(
            &config,
            Some("demo"),
            None,
            Some(&Stage::Sandbox),
            PathBuf::from("/tmp"),
        )
        .unwrap();
        assert_eq!(ctx.stage, Stage::Sandbox);
        // Refresh must still use the tenant that issued the stored token.
        assert_eq!(ctx.auth_stage, Stage::Prod);
    }

    #[test]
    fn test_build_ctx_stage_override_combined_with_auth_override() {
        use cli::Stage;
        use config::{Config, Profile};

        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Dev,
                default: None,
            },
        );
        config.profiles.insert(
            "service".to_string(),
            Profile {
                organization: "service".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );

        let ctx = build_ctx(
            &config,
            Some("demo"),
            Some("service"),
            Some(&Stage::Local),
            PathBuf::from("/tmp"),
        )
        .unwrap();
        assert_eq!(ctx.stage, Stage::Local);
        assert_eq!(ctx.auth_stage, Stage::Prod);
        assert_eq!(ctx.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_build_ctx_without_stage_override_uses_profile_stage() {
        use cli::Stage;
        use config::{Config, Profile};

        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Qa,
                default: None,
            },
        );

        let ctx = build_ctx(&config, Some("demo"), None, None, PathBuf::from("/tmp")).unwrap();
        assert_eq!(ctx.stage, Stage::Qa);
        assert_eq!(ctx.base_url, "https://demonstration.roundingwell.com/api");
    }

    #[test]
    fn test_check_stage_compatible_allows_login_without_override() {
        assert!(check_stage_compatible(&login_cmd(), None).is_ok());
    }

    #[test]
    fn test_check_stage_compatible_rejects_login_with_override() {
        let err = check_stage_compatible(&login_cmd(), Some(&cli::Stage::Dev)).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--stage"));
        assert!(msg.contains("auth login"));
    }

    #[test]
    fn test_check_stage_compatible_rejects_logout_with_override() {
        let err = check_stage_compatible(&logout_cmd(), Some(&cli::Stage::Dev)).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--stage"));
        assert!(msg.contains("auth logout"));
    }

    #[test]
    fn test_check_stage_compatible_allows_status_with_override() {
        assert!(check_stage_compatible(&status_cmd(), Some(&cli::Stage::Dev)).is_ok());
    }

    #[test]
    fn test_check_stage_compatible_allows_header_with_override() {
        assert!(check_stage_compatible(&header_cmd(), Some(&cli::Stage::Dev)).is_ok());
    }

    #[test]
    fn test_check_stage_compatible_allows_other_commands_with_override() {
        assert!(check_stage_compatible(&Commands::Update, Some(&cli::Stage::Dev)).is_ok());
    }
}
