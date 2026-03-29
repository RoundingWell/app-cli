mod api;
mod auth_cache;
mod cli;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;

use api::resolve_api;
use cli::{AuthCommands, Cli, Commands, ProfilesCommands};
use config::{load_config, resolve_profile};
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
    let mut config = load_config()?;

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
            commands::profile::set_default(&profile_args.name, &mut config, out)?;
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
                    out,
                )?;
            }
        },
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
