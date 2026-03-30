mod api;
mod auth_cache;
mod cli;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;

use api::resolve_api;
use cli::{AuthCommands, Cli, CliniciansCommands, Commands, ProfilesCommands};
use config::{config_path, load_config, resolve_profile};
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

async fn run(cli: Cli, out: &Output) -> Result<()> {
    let cfg_path = config_path()?;
    let mut config = load_config(&cfg_path)?;

    match cli.command {
        Commands::Auth(auth_args) => {
            let (profile, organization, stage) = resolve_profile(&config, cli.profile.as_deref())?;
            match auth_args.command {
                AuthCommands::Login => {
                    commands::auth::login(&profile, &organization, &stage, out).await?;
                }
                AuthCommands::Status => {
                    commands::auth::status(&profile, &organization, &stage, out)?;
                }
                AuthCommands::Header => {
                    commands::auth::header(&organization, &stage, out).await?;
                }
                AuthCommands::Logout => {
                    commands::auth::logout(&profile, &organization, &stage, out)?;
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
            let (_profile, organization, stage) = resolve_profile(&config, cli.profile.as_deref())?;
            let base_url = resolve_api(&organization, &stage);
            match clinician_args.command {
                CliniciansCommands::Assign(args) => {
                    commands::clinicians::assign(
                        &base_url,
                        &organization,
                        &stage,
                        &args.target,
                        &args.role,
                        out,
                    )
                    .await?;
                }
                CliniciansCommands::Enable(args) => {
                    commands::clinicians::enable(
                        &base_url,
                        &organization,
                        &stage,
                        &args.target,
                        out,
                    )
                    .await?;
                }
                CliniciansCommands::Disable(args) => {
                    commands::clinicians::disable(
                        &base_url,
                        &organization,
                        &stage,
                        &args.target,
                        out,
                    )
                    .await?;
                }
            }
        }
        Commands::Api(api_args) => {
            let (_profile, organization, stage) = resolve_profile(&config, cli.profile.as_deref())?;
            let base_url = resolve_api(&organization, &stage);
            commands::api::run(
                &base_url,
                &organization,
                &stage,
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
