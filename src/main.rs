use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use rw::api::resolve_api;
use rw::cli::{
    self, ArtifactsCommands, AuthCommands, Cli, CliniciansCommands, Commands, ConfigCommands,
    ConfigDefaultCommands, ConfigProfileCommands, ConfigUpdatesCommands, RolesCommands,
    SkillsCommands, TeamsCommands, WorkspacesCommands,
};
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
    config_dir: PathBuf,
) -> Result<AppContext> {
    let (profile, organization, stage) = resolve_profile(config, profile)?;
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

    match cli.command {
        Commands::Artifacts(artifacts_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            match artifacts_args.command {
                ArtifactsCommands::List(args) => {
                    commands::artifacts::list(
                        &ctx,
                        &args.artifact_type,
                        &args.path,
                        &args.term,
                        out,
                    )
                    .await?;
                }
            }
        }
        Commands::Auth(auth_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            match auth_args.command {
                AuthCommands::Login => {
                    commands::auth::login(&ctx, out).await?;
                }
                AuthCommands::Status => {
                    commands::auth::status(&ctx, out)?;
                }
                AuthCommands::Header => {
                    commands::auth::header(&ctx, out).await?;
                }
                AuthCommands::Logout => {
                    commands::auth::logout(&ctx, out)?;
                }
            }
        }
        Commands::Clinicians(clinician_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            match clinician_args.command {
                CliniciansCommands::Assign(args) => {
                    commands::clinicians::assign(&ctx, &args.target, &args.team, out).await?;
                }
                CliniciansCommands::Grant(args) => {
                    commands::clinicians::grant(&ctx, &args.target, &args.role, out).await?;
                }
                CliniciansCommands::Enable(args) => {
                    commands::clinicians::enable(&ctx, &args.target, out).await?;
                }
                CliniciansCommands::Disable(args) => {
                    commands::clinicians::disable(&ctx, &args.target, out).await?;
                }
                CliniciansCommands::Prepare(args) => {
                    commands::clinicians::prepare(&ctx, &args.target, out).await?;
                }
                CliniciansCommands::Register(args) => {
                    commands::clinicians::register(
                        &ctx,
                        &args.email,
                        &args.name,
                        args.role.as_deref(),
                        args.team.as_deref(),
                        out,
                    )
                    .await?;
                }
                CliniciansCommands::Show(args) => {
                    commands::clinicians::show(&ctx, &args.target, out).await?;
                }
                CliniciansCommands::Update(args) => {
                    commands::clinicians::update(
                        &ctx,
                        &args.target,
                        &args.field,
                        args.value.as_deref(),
                        out,
                    )
                    .await?;
                }
            }
        }
        Commands::Teams(teams_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            match teams_args.command {
                TeamsCommands::List(_) => {
                    commands::teams::list(&ctx, out).await?;
                }
                TeamsCommands::Show(args) => {
                    commands::teams::show(&ctx, &args.target, out).await?;
                }
            }
        }
        Commands::Roles(roles_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            match roles_args.command {
                RolesCommands::List(_) => {
                    commands::roles::list(&ctx, out).await?;
                }
                RolesCommands::Show(args) => {
                    commands::roles::show(&ctx, &args.target, out).await?;
                }
            }
        }
        Commands::Workspaces(workspaces_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            match workspaces_args.command {
                WorkspacesCommands::List(_) => {
                    commands::workspaces::list(&ctx, out).await?;
                }
                WorkspacesCommands::Show(args) => {
                    commands::workspaces::show(&ctx, &args.target, out).await?;
                }
            }
        }
        Commands::Api(api_args) => {
            let ctx = build_ctx(
                &config,
                cli.profile.as_deref(),
                cli.auth.as_deref(),
                config_dir,
            )?;
            commands::api::run(
                &ctx,
                &api_args.endpoint,
                &api_args.method,
                &api_args.headers,
                &api_args.fields,
                api_args.jq.as_deref(),
                api_args.raw,
            )
            .await?;
        }
        Commands::Update => {
            commands::update::run(out).await?;
        }
        Commands::Config(config_args) => match config_args.command {
            ConfigCommands::Profile(profile_args) => match profile_args.command {
                ConfigProfileCommands::List => {
                    commands::config::profile_list(&config, out);
                }
                ConfigProfileCommands::Show => {
                    commands::config::profile_show(&config, &config_dir, out)?;
                }
                ConfigProfileCommands::Use(args) => {
                    commands::config::profile_use(&args.name, &mut config, &cfg_path, out)?;
                }
                ConfigProfileCommands::Set(args) => {
                    commands::config::profile_set(args, &mut config, &cfg_path, out)?;
                }
                ConfigProfileCommands::Rm(args) => {
                    commands::config::profile_rm(args, &mut config, &cfg_path, &config_dir, out)?;
                }
                ConfigProfileCommands::Add(args) => {
                    commands::config::profile_add(args, &mut config, &cfg_path, out)?;
                }
                ConfigProfileCommands::Auth(args) => {
                    commands::config::profile_auth(args, &config, &config_dir, out)?;
                }
            },
            ConfigCommands::Updates(updates_args) => match updates_args.command {
                ConfigUpdatesCommands::Show => {
                    commands::config::updates_show(&config, out);
                }
                ConfigUpdatesCommands::Enable => {
                    commands::config::updates_enable(&mut config, &cfg_path, out)?;
                }
                ConfigUpdatesCommands::Disable => {
                    commands::config::updates_disable(&mut config, &cfg_path, out)?;
                }
            },
            ConfigCommands::Default(default_args) => match default_args.command {
                ConfigDefaultCommands::Set(args) => {
                    commands::config::default_set(
                        &args.key,
                        &args.value,
                        cli.profile.as_deref(),
                        &mut config,
                        &cfg_path,
                        out,
                    )?;
                }
                ConfigDefaultCommands::Get(args) => {
                    commands::config::default_get(&args.key, cli.profile.as_deref(), &config, out)?;
                }
                ConfigDefaultCommands::Rm(args) => {
                    commands::config::default_rm(
                        &args.key,
                        cli.profile.as_deref(),
                        &mut config,
                        &cfg_path,
                        out,
                    )?;
                }
                ConfigDefaultCommands::List => {
                    commands::config::default_list(cli.profile.as_deref(), &config, out)?;
                }
            },
        },
        Commands::Skills(skills_args) => match skills_args.command {
            SkillsCommands::Install(args) => {
                commands::skills::run_install(args.local, args.no_clobber, out)?;
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli::AuthArgs;

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

        let ctx = build_ctx(&config, Some("demo"), None, PathBuf::from("/tmp")).unwrap();
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
            PathBuf::from("/tmp"),
        )
        .unwrap();
        assert_eq!(ctx.stage, Stage::Dev);
        assert_eq!(ctx.auth_stage, Stage::Prod);
    }
}
