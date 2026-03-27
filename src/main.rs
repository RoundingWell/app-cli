mod cli;
mod commands;
mod config;
mod domain;

use anyhow::Result;
use clap::Parser;

use cli::{AuthCommands, Cli, Commands};
use config::{load_config, resolve_org_and_stage};
use domain::resolve_domain;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = load_config()?;

    let (organization, stage) = resolve_org_and_stage(
        &config,
        &cli.organization,
        &cli.stage,
        cli.profile.as_deref(),
    )?;

    match cli.command {
        Commands::Auth(auth_args) => match auth_args.command {
            AuthCommands::Login => {
                commands::auth::login(&organization).await?;
            }
            AuthCommands::Status => {
                commands::auth::status(&organization)?;
            }
            AuthCommands::Logout => {
                commands::auth::logout(&organization)?;
            }
        },
        Commands::Api(api_args) => {
            let base_url = resolve_domain(&organization, &stage);
            commands::api::run(
                &config,
                &base_url,
                &organization,
                &api_args.endpoint,
                &api_args.method,
                &api_args.headers,
                &api_args.fields,
                api_args.raw,
            )
            .await?;
        }
    }

    Ok(())
}
