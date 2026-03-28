mod api;
mod auth_cache;
mod cli;
mod commands;
mod config;

use anyhow::Result;
use clap::Parser;

use api::resolve_api;
use cli::{AuthCommands, Cli, Commands, Stage};
use config::{load_config, resolve_profile, save_config, Profile};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = load_config()?;

    match cli.command {
        Commands::Auth(auth_args) => {
            let (profile, organization, stage) = resolve_profile(&config, cli.profile.as_deref())?;
            match auth_args.command {
                AuthCommands::Login => {
                    commands::auth::login(&profile, &organization, &stage).await?;
                }
                AuthCommands::Status => {
                    commands::auth::status(&profile, &organization, &stage)?;
                }
                AuthCommands::Header => {
                    commands::auth::header(&organization, &stage).await?;
                }
                AuthCommands::Logout => {
                    commands::auth::logout(&profile, &organization, &stage)?;
                }
            }
        }
        Commands::Profile(profile_args) => {
            let name = &profile_args.name;
            if !config.profiles.contains_key(name.as_str()) {
                use std::io::{BufRead, Write};
                let stdin = std::io::stdin();
                let stdout = std::io::stdout();

                let organization = loop {
                    print!("Organization slug: ");
                    stdout.lock().flush()?;
                    let mut line = String::new();
                    if stdin.lock().read_line(&mut line)? == 0 {
                        anyhow::bail!("unexpected end of input");
                    }
                    match cli::validate_slug(line.trim()) {
                        Ok(s) => break s,
                        Err(e) => eprintln!("{}", e),
                    }
                };

                let stage = loop {
                    print!("Stage [prod, sandbox, qa, dev, local]: ");
                    stdout.lock().flush()?;
                    let mut line = String::new();
                    if stdin.lock().read_line(&mut line)? == 0 {
                        anyhow::bail!("unexpected end of input");
                    }
                    match line.trim() {
                        "prod" => break Stage::Prod,
                        "sandbox" => break Stage::Sandbox,
                        "qa" => break Stage::Qa,
                        "dev" => break Stage::Dev,
                        "local" => break Stage::Local,
                        other => eprintln!(
                            "\"{}\" is not a valid stage; must be one of: prod, sandbox, qa, dev, local",
                            other
                        ),
                    }
                };

                config.profiles.insert(
                    name.clone(),
                    Profile {
                        organization,
                        stage,
                    },
                );
            }
            config.default = Some(name.clone());
            save_config(&config)?;
            println!("Default profile set to \"{}\".", name);
        }
        Commands::Profiles => {
            let mut names: Vec<&String> = config.profiles.keys().collect();
            names.sort();
            for name in names {
                if config.default.as_deref() == Some(name) {
                    println!("* {}", name);
                } else {
                    println!("  {}", name);
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
