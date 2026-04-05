mod api;
mod auth_cache;
mod cli;
mod commands;
mod config;
mod migration;
mod output;
mod version_check;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use api::resolve_api;
use cli::{
    AuthCommands, Cli, CliniciansCommands, Commands, ConfigCommands, ConfigProfileCommands,
    ConfigUpdatesCommands,
};
use config::{config_path, default_config_dir, load_config, resolve_profile, AppContext};
use output::Output;

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
    config_dir: PathBuf,
) -> Result<AppContext> {
    let (profile, organization, stage) = resolve_profile(config, profile)?;
    let base_url = resolve_api(&organization, &stage);
    Ok(AppContext {
        config_dir,
        profile,
        stage,
        base_url,
    })
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
    // or when the user is explicitly running `rw config` (they may be disabling
    // auto updates).
    if !matches!(cli.command, Commands::Update | Commands::Config(_)) {
        version_check::check_and_update(&config_dir, &mut config, &cfg_path, out).await;
    }

    match cli.command {
        Commands::Auth(auth_args) => {
            let ctx = build_ctx(&config, cli.profile.as_deref(), config_dir)?;
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
            let ctx = build_ctx(&config, cli.profile.as_deref(), config_dir)?;
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
        Commands::Api(api_args) => {
            let ctx = build_ctx(&config, cli.profile.as_deref(), config_dir)?;
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
        },
    }

    Ok(())
}
