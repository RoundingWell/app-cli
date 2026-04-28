use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::cli::Stage;

/// Resolved application context: the config directory plus the active profile's values.
pub struct AppContext {
    pub config_dir: PathBuf,
    pub profile: String,
    /// Profile whose credentials should be used. Defaults to `profile` unless
    /// overridden by `--auth`.
    pub auth_profile: String,
    pub stage: Stage,
    /// Stage of the auth profile. Equals `stage` unless `--auth` selects a
    /// profile on a different stage; used to pick the correct WorkOS endpoint
    /// when refreshing the borrowed profile's token.
    pub auth_stage: Stage,
    pub base_url: String,
    pub defaults: BTreeMap<String, String>,
}

/// Root configuration file structure for `~/.config/rw/config.json`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "default", default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
}

/// A named profile linking a profile name to an organization + stage.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub organization: String,
    pub stage: Stage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<BTreeMap<String, String>>,
}

/// Returns the default config directory: `~/.config/rw`.
pub fn default_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".config").join("rw"))
}

/// Returns the path to the config file within `config_dir`.
pub fn config_path(config_dir: &Path) -> PathBuf {
    config_dir.join("config.json")
}

/// Loads the configuration from disk, returning a default empty config if the
/// file does not exist yet. Automatically migrates the legacy `default_profile`
/// key to `default` and rewrites the file if migration was needed.
pub fn load_config(path: &std::path::Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("could not read config file: {}", path.display()))?;
    let mut value: serde_json::Value = serde_json::from_str(&contents)
        .with_context(|| format!("could not parse config file: {}", path.display()))?;

    // Migrate `default_profile` -> `default` if the old key is present.
    let migrated = if let Some(obj) = value.as_object_mut() {
        if let Some(v) = obj.remove("default_profile") {
            if !obj.contains_key("default") {
                obj.insert("default".to_string(), v);
            }
            true
        } else {
            false
        }
    } else {
        false
    };

    let config: Config = serde_json::from_value(value)
        .with_context(|| format!("could not parse config file: {}", path.display()))?;

    if migrated {
        save_config_to(&config, path)?;
    }

    Ok(config)
}

/// Persists the configuration to the given path, creating parent directories as needed.
pub fn save_config_to(config: &Config, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create config directory: {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(config).context("could not serialize config")?;
    write_atomic::write_file(path, contents.as_bytes())
        .with_context(|| format!("could not write config file: {}", path.display()))?;
    Ok(())
}

/// Resolves the effective profile name, organization, and stage.
/// Uses `--profile` if given, then `default` from config.
/// Returns an error if neither is set.
pub fn resolve_profile(config: &Config, profile: Option<&str>) -> Result<(String, String, Stage)> {
    let effective_profile = profile.or(config.default.as_deref());
    if let Some(name) = effective_profile {
        let p = config
            .profiles
            .get(name)
            .with_context(|| format!("profile \"{}\" not found in config", name))?;
        Ok((name.to_string(), p.organization.clone(), p.stage.clone()))
    } else {
        anyhow::bail!(
            "no profile selected; run `rw config profile use <name>` to set a default, or pass --profile"
        )
    }
}

/// Resolves the profile whose stored credentials should be used for the
/// invocation. Returns `auth.unwrap_or(profile)` after verifying the override
/// (when present) names a known profile in `config`.
pub fn resolve_auth_profile(config: &Config, profile: &str, auth: Option<&str>) -> Result<String> {
    match auth {
        None => Ok(profile.to_string()),
        Some(name) => {
            if !config.profiles.contains_key(name) {
                anyhow::bail!("profile \"{}\" not found in config", name);
            }
            Ok(name.to_string())
        }
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
                default: None,
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
                default: None,
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
                default: None,
            },
        );
        config.default = Some("demo".to_string());
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

    #[test]
    fn test_default_key_serialized_first() {
        let mut config = Config {
            default: Some("demo".to_string()),
            ..Default::default()
        };
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );
        let json = serde_json::to_string_pretty(&config).unwrap();
        let default_pos = json.find("\"default\"").unwrap();
        let profiles_pos = json.find("\"profiles\"").unwrap();
        assert!(
            default_pos < profiles_pos,
            "\"default\" should appear before \"profiles\""
        );
    }

    #[test]
    fn test_migrate_default_profile_key() {
        let legacy_json = r#"{"profiles":{"demo":{"organization":"demonstration","stage":"prod"}},"default_profile":"demo"}"#;
        let mut value: serde_json::Value = serde_json::from_str(legacy_json).unwrap();
        if let Some(obj) = value.as_object_mut() {
            if let Some(v) = obj.remove("default_profile") {
                obj.insert("default".to_string(), v);
            }
        }
        let config: Config = serde_json::from_value(value).unwrap();
        assert_eq!(config.default.as_deref(), Some("demo"));
        assert!(config.profiles.contains_key("demo"));
    }

    #[test]
    fn test_migrate_default_profile_does_not_overwrite_default() {
        // If both keys are present, the existing `default` value takes precedence.
        let json = r#"{"default":"mercy","default_profile":"demo","profiles":{}}"#;
        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        if let Some(obj) = value.as_object_mut() {
            if let Some(v) = obj.remove("default_profile") {
                if !obj.contains_key("default") {
                    obj.insert("default".to_string(), v);
                }
            }
        }
        let config: Config = serde_json::from_value(value).unwrap();
        assert_eq!(config.default.as_deref(), Some("mercy"));
    }

    #[test]
    fn test_profile_deserializes_without_default_field() {
        let json = r#"{"organization":"mercy","stage":"prod"}"#;
        let profile: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.organization, "mercy");
        assert!(profile.default.is_none());
    }

    #[test]
    fn test_profile_deserializes_with_default_field() {
        let json = r#"{"organization":"mercy","stage":"prod","default":{"role":"physician","team":"ICU"}}"#;
        let profile: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(
            profile
                .default
                .as_ref()
                .unwrap()
                .get("role")
                .map(String::as_str),
            Some("physician")
        );
        assert_eq!(
            profile
                .default
                .as_ref()
                .unwrap()
                .get("team")
                .map(String::as_str),
            Some("ICU")
        );
    }

    #[test]
    fn test_resolve_auth_profile_none_returns_active() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );
        let result = resolve_auth_profile(&config, "demo", None).unwrap();
        assert_eq!(result, "demo");
    }

    #[test]
    fn test_resolve_auth_profile_some_returns_override() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );
        config.profiles.insert(
            "mercy".to_string(),
            Profile {
                organization: "mercy".to_string(),
                stage: Stage::Sandbox,
                default: None,
            },
        );
        let result = resolve_auth_profile(&config, "demo", Some("mercy")).unwrap();
        assert_eq!(result, "mercy");
    }

    #[test]
    fn test_resolve_auth_profile_unknown_errors() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );
        let err = resolve_auth_profile(&config, "demo", Some("nope")).unwrap_err();
        assert!(err.to_string().contains("\"nope\""));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_resolve_auth_profile_same_as_active_is_silent_noop() {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );
        let result = resolve_auth_profile(&config, "demo", Some("demo")).unwrap();
        assert_eq!(result, "demo");
    }
}
