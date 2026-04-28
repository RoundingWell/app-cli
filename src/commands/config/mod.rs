//! `rw config` subcommands.
//!
//! Each top-level subcommand (`profile`, `updates`, `default`, `doctor`) lives
//! in its own file. Interactive prompt helpers are in `prompts`.

mod default;
mod doctor;
mod profile;
mod updates;

pub use default::{default_get, default_list, default_rm, default_set};
pub use profile::{
    profile_add, profile_auth, profile_list, profile_rm, profile_set, profile_show, profile_use,
};
pub use updates::{updates_disable, updates_enable, updates_show};

use anyhow::Result;
use std::path::Path;

use crate::cli::{
    ConfigArgs, ConfigCommands, ConfigDefaultCommands, ConfigProfileCommands, ConfigUpdatesCommands,
};
use crate::config::Config;
use crate::output::Output;

pub async fn dispatch(
    args: ConfigArgs,
    config: &mut Config,
    cfg_path: &Path,
    config_dir: &Path,
    profile_override: Option<&str>,
    out: &Output,
) -> Result<()> {
    match args.command {
        ConfigCommands::Doctor => doctor::doctor(config, config_dir, profile_override, out).await,
        ConfigCommands::Profile(profile_args) => match profile_args.command {
            ConfigProfileCommands::List => {
                profile_list(config, out);
                Ok(())
            }
            ConfigProfileCommands::Show => {
                profile_show(config, config_dir, out)?;
                Ok(())
            }
            ConfigProfileCommands::Use(a) => profile_use(&a.name, config, cfg_path, out),
            ConfigProfileCommands::Set(a) => profile_set(a, config, cfg_path, out),
            ConfigProfileCommands::Rm(a) => profile_rm(a, config, cfg_path, config_dir, out),
            ConfigProfileCommands::Add(a) => profile_add(a, config, cfg_path, out),
            ConfigProfileCommands::Auth(a) => profile_auth(a, config, config_dir, out),
        },
        ConfigCommands::Updates(updates_args) => match updates_args.command {
            ConfigUpdatesCommands::Show => {
                updates_show(config, out);
                Ok(())
            }
            ConfigUpdatesCommands::Enable => updates_enable(config, cfg_path, out),
            ConfigUpdatesCommands::Disable => updates_disable(config, cfg_path, out),
        },
        ConfigCommands::Default(default_args) => match default_args.command {
            ConfigDefaultCommands::Set(a) => {
                default_set(&a.key, &a.value, profile_override, config, cfg_path, out)
            }
            ConfigDefaultCommands::Get(a) => default_get(&a.key, profile_override, config, out),
            ConfigDefaultCommands::Rm(a) => {
                default_rm(&a.key, profile_override, config, cfg_path, out)
            }
            ConfigDefaultCommands::List => default_list(profile_override, config, out),
        },
    }
}
