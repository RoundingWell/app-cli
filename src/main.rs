mod api;
mod auth_cache;
mod cli;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use api::resolve_api;
use cli::{AuthCommands, BasicCommands, Cli, CliniciansCommands, Commands, ProfilesCommands};
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
        organization,
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
        Commands::Basic(basic_args) => {
            let ctx = build_ctx(&config, cli.profile.as_deref(), config_dir)?;
            match basic_args.command {
                BasicCommands::Set(args) => {
                    commands::basic::set(&ctx, args.username, args.password, out)?;
                }
            }
        }
        Commands::Profile(profile_args) => {
            commands::profile::set_default(&profile_args.name, &mut config, &cfg_path, out)?;
        }
        Commands::Profiles(profiles_args) => match profiles_args.command {
            None => {
                commands::profile::list(&config, out);
            }
            Some(ProfilesCommands::Add(args)) => {
                commands::profile::add(
                    &args.name,
                    args.organization,
                    args.stage,
                    &mut config,
                    &cfg_path,
                    out,
                )?;
            }
            Some(ProfilesCommands::Rm(args)) => {
                commands::profile::rm(&args.name, &mut config, &cfg_path, out)?;
            }
        },
        Commands::Clinicians(clinician_args) => {
            let ctx = build_ctx(&config, cli.profile.as_deref(), config_dir)?;
            match clinician_args.command {
                CliniciansCommands::Assign(args) => {
                    commands::clinicians::assign(&ctx, &args.target, &args.role, out).await?;
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
    }

    Ok(())
}
