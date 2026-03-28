use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli::Stage;

/// Root configuration file structure for `~/.config/rw/profiles.json`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
}

/// A named profile linking a profile name to an organization + stage.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub organization: String,
    pub stage: Stage,
}

/// Returns the path to the config file: `~/.config/rw/profiles.json`.
pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".config").join("rw").join("profiles.json"))
}

/// Loads the configuration from disk, returning a default empty config if the
/// file does not exist yet.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read config file: {}", path.display()))?;
    let config: Config = serde_json::from_str(&contents)
        .with_context(|| format!("could not parse config file: {}", path.display()))?;
    Ok(config)
}

/// Persists the configuration to `~/.config/rw/profiles.json`, creating
/// parent directories as needed.
pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create config directory: {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(config).context("could not serialize config")?;
    std::fs::write(&path, contents)
        .with_context(|| format!("could not write config file: {}", path.display()))?;
    Ok(())
}

/// Resolves the effective profile name, organization, and stage.
/// Uses `--profile` if given, then `default_profile` from config.
/// Returns an error if neither is set.
pub fn resolve_profile(config: &Config, profile: Option<&str>) -> Result<(String, String, Stage)> {
    let effective_profile = profile.or(config.default_profile.as_deref());
    if let Some(name) = effective_profile {
        let p = config
            .profiles
            .get(name)
            .with_context(|| format!("profile \"{}\" not found in config", name))?;
        Ok((name.to_string(), p.organization.clone(), p.stage.clone()))
    } else {
        anyhow::bail!(
            "no profile selected; run `rw profile <name>` to set a default, or pass --profile"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
            },
        );
        let json = serde_json::to_string_pretty(&config).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.profiles["demo"].organization, "demonstration");
    }

    #[test]
    fn test_resolve_organization_profile() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
            },
        );
        let (profile, organization, stage) = resolve_profile(&config, Some("demo")).unwrap();
        assert_eq!(profile, "demo");
        assert_eq!(organization, "demonstration");
        assert_eq!(stage, Stage::Prod);
    }

    #[test]
    fn test_resolve_default_profile() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Sandbox,
            },
        );
        config.default_profile = Some("demo".to_string());
        let (profile, organization, stage) = resolve_profile(&config, None).unwrap();
        assert_eq!(profile, "demo");
        assert_eq!(organization, "demonstration");
        assert_eq!(stage, Stage::Sandbox);
    }

    #[test]
    fn test_resolve_no_profile_errors() {
        let config = Config::default();
        assert!(resolve_profile(&config, None).is_err());
    }
}
