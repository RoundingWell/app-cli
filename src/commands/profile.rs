use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::cli::{validate_slug, Stage};
use crate::config::{save_config_to, Config, Profile};
use crate::output::{CommandOutput, Output};

/// Run `rw profile <name>` – set the named profile as default. Errors if the profile does not exist.
pub fn set_default(
    name: &str,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    if !config.profiles.contains_key(name) {
        anyhow::bail!(
            "profile '{}' does not exist; use 'rw profiles add {}' to add it",
            name,
            name
        );
    }
    config.default = Some(name.to_string());
    save_config_to(config, config_path)?;
    out.print(&SetDefaultOutput {
        name: name.to_string(),
    });
    Ok(())
}

fn prompt_organization() -> Result<String> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    loop {
        print!("Organization slug: ");
        stdout.lock().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            anyhow::bail!("unexpected end of input");
        }
        match validate_slug(line.trim()) {
            Ok(s) => return Ok(s),
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn prompt_stage() -> Result<Stage> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    loop {
        print!("Stage [prod, sandbox, qa, dev, local]: ");
        stdout.lock().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            anyhow::bail!("unexpected end of input");
        }
        match line.trim() {
            "prod" => return Ok(Stage::Prod),
            "sandbox" => return Ok(Stage::Sandbox),
            "qa" => return Ok(Stage::Qa),
            "dev" => return Ok(Stage::Dev),
            "local" => return Ok(Stage::Local),
            other => eprintln!(
                "'{}' is not a valid stage; must be one of: prod, sandbox, qa, dev, local",
                other
            ),
        }
    }
}

/// Run `rw profiles add <name>` – add a new profile.
///
/// `organization` and `stage` may be provided via flags; any missing values are collected
/// interactively.  If `--json` is active and interactive mode would be required, an error is
/// returned instead.
pub fn add(
    name: &str,
    organization: Option<String>,
    stage: Option<Stage>,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    if config.profiles.contains_key(name) {
        anyhow::bail!("profile '{}' already exists", name);
    }

    if out.json && (organization.is_none() || stage.is_none()) {
        anyhow::bail!(
            "cannot use interactive mode with --json; provide --organization and --stage"
        );
    }

    let organization = organization.map(Ok).unwrap_or_else(prompt_organization)?;
    let stage = stage.map(Ok).unwrap_or_else(prompt_stage)?;

    config.profiles.insert(
        name.to_string(),
        Profile {
            organization: organization.clone(),
            stage: stage.clone(),
        },
    );
    save_config_to(config, config_path)?;
    out.print(&AddOutput {
        name: name.to_string(),
        organization,
        stage,
    });
    Ok(())
}

/// Run `rw profiles rm <name>` – remove a profile. Clears the default if it was the removed profile.
pub fn rm(name: &str, config: &mut Config, config_path: &Path, out: &Output) -> Result<()> {
    if !config.profiles.contains_key(name) {
        anyhow::bail!("profile '{}' does not exist", name);
    }
    config.profiles.remove(name);
    if config.default.as_deref() == Some(name) {
        config.default = None;
    }
    save_config_to(config, config_path)?;
    out.print(&RmOutput {
        name: name.to_string(),
    });
    Ok(())
}

/// Run `rw profiles` – list all available profiles.
pub fn list(config: &Config, out: &Output) {
    let mut names: Vec<String> = config.profiles.keys().cloned().collect();
    names.sort();
    out.print(&ProfilesOutput {
        profiles: names,
        default: config.default.clone(),
    });
}

#[derive(Serialize)]
pub struct SetDefaultOutput {
    pub name: String,
}

impl CommandOutput for SetDefaultOutput {
    fn plain(&self) -> String {
        format!("Default profile set to '{}'.", self.name)
    }
}

#[derive(Serialize)]
pub struct AddOutput {
    pub name: String,
    pub organization: String,
    pub stage: Stage,
}

impl CommandOutput for AddOutput {
    fn plain(&self) -> String {
        format!("Profile '{}' created.", self.name)
    }
}

#[derive(Serialize)]
pub struct RmOutput {
    pub name: String,
}

impl CommandOutput for RmOutput {
    fn plain(&self) -> String {
        format!("Profile '{}' removed.", self.name)
    }
}

#[derive(Serialize)]
pub struct ProfilesOutput {
    pub profiles: Vec<String>,
    pub default: Option<String>,
}

impl CommandOutput for ProfilesOutput {
    fn plain(&self) -> String {
        self.profiles
            .iter()
            .map(|name| {
                if self.default.as_deref() == Some(name) {
                    format!("* {}", name)
                } else {
                    format!("  {}", name)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::output::Output;
    use std::collections::BTreeMap;

    fn out_plain() -> Output {
        Output { json: false }
    }

    fn out_json() -> Output {
        Output { json: true }
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
            },
        );
        Config {
            version: None,
            default: None,
            profiles,
            auto_update: None,
        }
    }

    #[test]
    fn test_set_default_output_plain() {
        let output = SetDefaultOutput {
            name: "demo".to_string(),
        };
        assert_eq!(output.plain(), "Default profile set to 'demo'.");
    }

    #[test]
    fn test_set_default_errors_when_profile_not_found() {
        let (_tmp, path) = tmp_path();
        let mut config = Config {
            version: None,
            default: None,
            profiles: BTreeMap::new(),
            auto_update: None,
        };
        let err = set_default("missing", &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
        assert!(err.to_string().contains("profiles add missing"));
    }

    #[test]
    fn test_profiles_output_plain_marks_default() {
        let output = ProfilesOutput {
            profiles: vec!["demo".to_string(), "sandbox".to_string()],
            default: Some("demo".to_string()),
        };
        assert_eq!(output.plain(), "* demo\n  sandbox");
    }

    #[test]
    fn test_profiles_output_plain_no_default() {
        let output = ProfilesOutput {
            profiles: vec!["demo".to_string()],
            default: None,
        };
        assert_eq!(output.plain(), "  demo");
    }

    #[test]
    fn test_profiles_output_json() {
        let output = ProfilesOutput {
            profiles: vec!["demo".to_string(), "sandbox".to_string()],
            default: Some("demo".to_string()),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["profiles"], serde_json::json!(["demo", "sandbox"]));
        assert_eq!(json["default"], "demo");
    }

    #[test]
    fn test_profiles_output_json_no_default() {
        let output = ProfilesOutput {
            profiles: vec![],
            default: None,
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["default"].is_null());
    }

    #[test]
    fn test_create_output_plain() {
        let output = AddOutput {
            name: "demo".to_string(),
            organization: "mercy".to_string(),
            stage: Stage::Prod,
        };
        assert_eq!(output.plain(), "Profile 'demo' created.");
    }

    #[test]
    fn test_create_output_json() {
        let output = AddOutput {
            name: "demo".to_string(),
            organization: "mercy".to_string(),
            stage: Stage::Sandbox,
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["name"], "demo");
        assert_eq!(json["organization"], "mercy");
        assert_eq!(json["stage"], "sandbox");
    }

    #[test]
    fn test_create_errors_when_profile_already_exists() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let err = add(
            "demo",
            Some("mercy".to_string()),
            Some(Stage::Prod),
            &mut config,
            &path,
            &out_plain(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_create_errors_with_json_flag_when_organization_missing() {
        let (_tmp, path) = tmp_path();
        let mut config = Config {
            version: None,
            default: None,
            profiles: BTreeMap::new(),
            auto_update: None,
        };
        let err = add(
            "demo",
            None,
            Some(Stage::Prod),
            &mut config,
            &path,
            &out_json(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("--json"));
    }

    #[test]
    fn test_create_errors_with_json_flag_when_stage_missing() {
        let (_tmp, path) = tmp_path();
        let mut config = Config {
            version: None,
            default: None,
            profiles: BTreeMap::new(),
            auto_update: None,
        };
        let err = add(
            "demo",
            Some("mercy".to_string()),
            None,
            &mut config,
            &path,
            &out_json(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("--json"));
    }

    #[test]
    fn test_rm_output_plain() {
        let output = RmOutput {
            name: "demo".to_string(),
        };
        assert_eq!(output.plain(), "Profile 'demo' removed.");
    }

    #[test]
    fn test_rm_output_json() {
        let output = RmOutput {
            name: "demo".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["name"], "demo");
    }

    #[test]
    fn test_rm_removes_profile() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        rm("demo", &mut config, &path, &out_plain()).unwrap();
        assert!(!config.profiles.contains_key("demo"));
    }

    #[test]
    fn test_rm_clears_default_when_removed() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        config.default = Some("demo".to_string());
        rm("demo", &mut config, &path, &out_plain()).unwrap();
        assert!(config.default.is_none());
    }

    #[test]
    fn test_rm_leaves_default_when_other_profile_removed() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        config.profiles.insert(
            "other".to_string(),
            Profile {
                organization: "other-org".to_string(),
                stage: Stage::Sandbox,
            },
        );
        config.default = Some("demo".to_string());
        rm("other", &mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.default.as_deref(), Some("demo"));
    }

    #[test]
    fn test_rm_errors_when_profile_not_found() {
        let (_tmp, path) = tmp_path();
        let mut config = Config {
            version: None,
            default: None,
            profiles: BTreeMap::new(),
            auto_update: None,
        };
        let err = rm("missing", &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}
