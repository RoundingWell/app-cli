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
    #[serde(default)]
    pub authentication: HashMap<String, AuthEntry>,
}

/// A named profile linking a profile name to an organization + stage.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub organization: String,
    pub stage: Stage,
}

/// Authentication credentials stored for an organization.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AuthEntry {
    Bearer { bearer: String },
    Basic { basic: BasicCredentials },
}

/// HTTP Basic credentials.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicCredentials {
    pub username: String,
    pub password: String,
}

/// Returns the path to the config file: `~/.config/rw/profiles.json`.
pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("could not determine config directory")?;
    Ok(config_dir.join("rw").join("profiles.json"))
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

/// Resolves the effective organization and stage, taking `--profile` into
/// account when specified.
pub fn resolve_org_and_stage(
    config: &Config,
    organization: &str,
    stage: &Stage,
    profile: Option<&str>,
) -> Result<(String, Stage)> {
    if let Some(name) = profile {
        let p = config
            .profiles
            .get(name)
            .with_context(|| format!("profile \"{}\" not found in config", name))?;
        Ok((p.organization.clone(), p.stage.clone()))
    } else {
        Ok((organization.to_string(), stage.clone()))
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
        config.authentication.insert(
            "demonstration".to_string(),
            AuthEntry::Bearer {
                bearer: "token123".to_string(),
            },
        );
        let json = serde_json::to_string_pretty(&config).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.profiles["demo"].organization, "demonstration");
        match &loaded.authentication["demonstration"] {
            AuthEntry::Bearer { bearer } => assert_eq!(bearer, "token123"),
            _ => panic!("expected bearer auth"),
        }
    }

    #[test]
    fn test_resolve_org_profile() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
            },
        );
        let (org, stage) =
            resolve_org_and_stage(&config, "other", &Stage::Dev, Some("demo")).unwrap();
        assert_eq!(org, "demonstration");
        assert_eq!(stage, Stage::Prod);
    }

    #[test]
    fn test_resolve_org_no_profile() {
        let config = Config::default();
        let (org, stage) =
            resolve_org_and_stage(&config, "myorg", &Stage::Sandbox, None).unwrap();
        assert_eq!(org, "myorg");
        assert_eq!(stage, Stage::Sandbox);
    }
}
