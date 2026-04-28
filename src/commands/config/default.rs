//! `rw config default` subcommands: set / get / list / rm.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::config::{save_config_to, Config};
use crate::output::{CommandOutput, Output};

const DEFAULT_ALLOWED_KEYS: &[&str] = &["role", "team"];

fn validate_default_key(key: &str) -> Result<()> {
    if DEFAULT_ALLOWED_KEYS.contains(&key) {
        Ok(())
    } else {
        anyhow::bail!(
            "unknown key '{}'; allowed keys are: {}",
            key,
            DEFAULT_ALLOWED_KEYS.join(", ")
        )
    }
}

fn resolve_profile_name<'a>(
    config: &'a Config,
    profile_override: Option<&'a str>,
) -> Result<&'a str> {
    profile_override
        .or(config.default.as_deref())
        .ok_or_else(|| {
            anyhow::anyhow!("no default profile configured; use 'rw config profile use <name>'")
        })
}

#[derive(Serialize)]
pub struct DefaultSetOutput {
    pub key: String,
    pub value: String,
}

impl CommandOutput for DefaultSetOutput {
    fn plain(&self) -> String {
        format!("Default '{}' set to '{}'.", self.key, self.value)
    }
}

#[derive(Serialize)]
pub struct DefaultGetOutput {
    pub key: String,
    pub value: String,
}

impl CommandOutput for DefaultGetOutput {
    fn plain(&self) -> String {
        self.value.clone()
    }
}

#[derive(Serialize)]
pub struct DefaultRmOutput {
    pub key: String,
}

impl CommandOutput for DefaultRmOutput {
    fn plain(&self) -> String {
        format!("Default '{}' removed.", self.key)
    }
}

#[derive(Serialize)]
pub struct DefaultListEntry {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct DefaultListOutput {
    pub defaults: Vec<DefaultListEntry>,
}

impl CommandOutput for DefaultListOutput {
    fn plain(&self) -> String {
        self.defaults
            .iter()
            .map(|e| format!("{}={}", e.key, e.value))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn default_set(
    key: &str,
    value: &str,
    profile: Option<&str>,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    validate_default_key(key)?;
    let profile_name = resolve_profile_name(config, profile)?.to_string();
    let profile = config
        .profiles
        .get_mut(&profile_name)
        .ok_or_else(|| anyhow::anyhow!("profile '{}' not found", profile_name))?;
    profile
        .default
        .get_or_insert_with(Default::default)
        .insert(key.to_string(), value.to_string());
    save_config_to(config, config_path)?;
    out.print(&DefaultSetOutput {
        key: key.to_string(),
        value: value.to_string(),
    });
    Ok(())
}

pub fn default_get(key: &str, profile: Option<&str>, config: &Config, out: &Output) -> Result<()> {
    validate_default_key(key)?;
    let profile_name = resolve_profile_name(config, profile)?;
    let profile = config
        .profiles
        .get(profile_name)
        .ok_or_else(|| anyhow::anyhow!("profile '{}' not found", profile_name))?;
    if let Some(value) = profile.default.as_ref().and_then(|d| d.get(key)) {
        out.print(&DefaultGetOutput {
            key: key.to_string(),
            value: value.clone(),
        });
    }
    Ok(())
}

pub fn default_list(profile: Option<&str>, config: &Config, out: &Output) -> Result<()> {
    let profile_name = resolve_profile_name(config, profile)?;
    let profile = config
        .profiles
        .get(profile_name)
        .ok_or_else(|| anyhow::anyhow!("profile '{}' not found", profile_name))?;
    let entries: Vec<DefaultListEntry> = profile
        .default
        .iter()
        .flat_map(|d| d.iter())
        .map(|(k, v)| DefaultListEntry {
            key: k.clone(),
            value: v.clone(),
        })
        .collect();
    if !entries.is_empty() {
        out.print(&DefaultListOutput { defaults: entries });
    }
    Ok(())
}

pub fn default_rm(
    key: &str,
    profile: Option<&str>,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    validate_default_key(key)?;
    let profile_name = resolve_profile_name(config, profile)?.to_string();
    let profile = config
        .profiles
        .get_mut(&profile_name)
        .ok_or_else(|| anyhow::anyhow!("profile '{}' not found", profile_name))?;
    let was_set = profile
        .default
        .as_mut()
        .and_then(|d| d.remove(key))
        .is_some();
    if profile.default.as_ref().is_some_and(|d| d.is_empty()) {
        profile.default = None;
    }
    if was_set {
        save_config_to(config, config_path)?;
        out.print(&DefaultRmOutput {
            key: key.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Stage;
    use crate::config::{Config, Profile};
    use std::collections::BTreeMap;

    fn out_plain() -> Output {
        Output { json: false }
    }

    fn tmp_path() -> (tempfile::NamedTempFile, std::path::PathBuf) {
        let f = tempfile::NamedTempFile::new().unwrap();
        let p = f.path().to_path_buf();
        (f, p)
    }

    fn config_with_profile(name: &str, org: &str, stage: Stage) -> Config {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            name.to_string(),
            Profile {
                organization: org.to_string(),
                stage,
                default: None,
            },
        );
        Config {
            version: None,
            default: None,
            profiles,
            auto_update: None,
        }
    }

    fn config_with_default_profile(name: &str) -> Config {
        let mut config = config_with_profile(name, "mercy", Stage::Prod);
        config.default = Some(name.to_string());
        config
    }

    /// Config with two profiles: `active` is the configured default and `other` is a second profile.
    fn config_with_two_profiles() -> Config {
        let mut config = config_with_default_profile("active");
        config.profiles.insert(
            "other".to_string(),
            Profile {
                organization: "mercy".to_string(),
                stage: Stage::Prod,
                default: None,
            },
        );
        config
    }

    #[test]
    fn test_default_set_valid_key_role() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        default_set("role", "physician", None, &mut config, &path, &out_plain()).unwrap();
        assert_eq!(
            config.profiles["demo"]
                .default
                .as_ref()
                .unwrap()
                .get("role")
                .map(String::as_str),
            Some("physician")
        );
    }

    #[test]
    fn test_default_set_valid_key_team() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        default_set("team", "ICU", None, &mut config, &path, &out_plain()).unwrap();
        assert_eq!(
            config.profiles["demo"]
                .default
                .as_ref()
                .unwrap()
                .get("team")
                .map(String::as_str),
            Some("ICU")
        );
    }

    #[test]
    fn test_default_set_unknown_key_returns_error() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        let err = default_set("foo", "bar", None, &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("unknown key"));
        assert!(err.to_string().contains("team"));
        assert!(err.to_string().contains("role"));
    }

    #[test]
    fn test_default_get_returns_value_when_set() {
        let mut config = config_with_default_profile("demo");
        config.profiles.get_mut("demo").unwrap().default =
            Some([("role".to_string(), "employee".to_string())].into());
        default_get("role", None, &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_get_no_output_when_unset() {
        let config = config_with_default_profile("demo");
        default_get("team", None, &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_get_unknown_key_returns_error() {
        let config = config_with_default_profile("demo");
        let err = default_get("foo", None, &config, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("unknown key"));
    }

    #[test]
    fn test_default_rm_removes_key_when_set() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        config.profiles.get_mut("demo").unwrap().default =
            Some([("role".to_string(), "employee".to_string())].into());
        default_rm("role", None, &mut config, &path, &out_plain()).unwrap();
        assert!(config.profiles["demo"]
            .default
            .as_ref()
            .map_or(true, |d| !d.contains_key("role")));
    }

    #[test]
    fn test_default_rm_no_error_when_unset() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        default_rm("team", None, &mut config, &path, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_rm_unknown_key_returns_error() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        let err = default_rm("foo", None, &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("unknown key"));
    }

    #[test]
    fn test_default_rm_sets_default_to_none_when_last_key_removed() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_default_profile("demo");
        config.profiles.get_mut("demo").unwrap().default =
            Some([("role".to_string(), "employee".to_string())].into());
        default_rm("role", None, &mut config, &path, &out_plain()).unwrap();
        assert!(config.profiles["demo"].default.is_none());
    }

    #[test]
    fn test_default_list_multiple_defaults() {
        let mut config = config_with_default_profile("demo");
        config.profiles.get_mut("demo").unwrap().default = Some(
            [
                ("role".to_string(), "employee".to_string()),
                ("team".to_string(), "NUR".to_string()),
            ]
            .into(),
        );
        default_list(None, &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_list_no_defaults() {
        let config = config_with_default_profile("demo");
        default_list(None, &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_list_one_default() {
        let mut config = config_with_default_profile("demo");
        config.profiles.get_mut("demo").unwrap().default =
            Some([("role".to_string(), "physician".to_string())].into());
        default_list(None, &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_set_uses_explicit_profile_override() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_two_profiles();
        default_set(
            "role",
            "nurse",
            Some("other"),
            &mut config,
            &path,
            &out_plain(),
        )
        .unwrap();
        assert_eq!(
            config.profiles["other"]
                .default
                .as_ref()
                .unwrap()
                .get("role")
                .map(String::as_str),
            Some("nurse")
        );
        assert!(config.profiles["active"]
            .default
            .as_ref()
            .map_or(true, |d| !d.contains_key("role")));
    }

    #[test]
    fn test_default_get_uses_explicit_profile_override() {
        let mut config = config_with_two_profiles();
        config.profiles.get_mut("other").unwrap().default =
            Some([("role".to_string(), "nurse".to_string())].into());
        config.profiles.get_mut("active").unwrap().default =
            Some([("role".to_string(), "physician".to_string())].into());
        default_get("role", Some("other"), &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_rm_uses_explicit_profile_override() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_two_profiles();
        config.profiles.get_mut("other").unwrap().default =
            Some([("role".to_string(), "nurse".to_string())].into());
        default_rm("role", Some("other"), &mut config, &path, &out_plain()).unwrap();
        assert!(config.profiles["other"]
            .default
            .as_ref()
            .map_or(true, |d| !d.contains_key("role")));
        config.profiles.get_mut("active").unwrap().default =
            Some([("role".to_string(), "physician".to_string())].into());
        assert_eq!(
            config.profiles["active"]
                .default
                .as_ref()
                .unwrap()
                .get("role")
                .map(String::as_str),
            Some("physician")
        );
    }

    #[test]
    fn test_default_list_uses_explicit_profile_override() {
        let mut config = config_with_two_profiles();
        config.profiles.get_mut("other").unwrap().default =
            Some([("team".to_string(), "ICU".to_string())].into());
        default_list(Some("other"), &config, &out_plain()).unwrap();
    }

    #[test]
    fn test_default_list_output_plain_sorted() {
        let output = DefaultListOutput {
            defaults: vec![
                DefaultListEntry {
                    key: "role".to_string(),
                    value: "employee".to_string(),
                },
                DefaultListEntry {
                    key: "team".to_string(),
                    value: "NUR".to_string(),
                },
            ],
        };
        let text = output.plain();
        assert_eq!(text, "role=employee\nteam=NUR");
    }

    #[test]
    fn test_default_get_output_plain_returns_value() {
        let output = DefaultGetOutput {
            key: "role".to_string(),
            value: "physician".to_string(),
        };
        assert_eq!(output.plain(), "physician");
    }

    #[test]
    fn test_default_get_output_json_serializes_key_and_value() {
        let output = DefaultGetOutput {
            key: "role".to_string(),
            value: "physician".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["key"], "role");
        assert_eq!(json["value"], "physician");
    }
}
