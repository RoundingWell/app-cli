use anyhow::Result;
use serde::Serialize;

use crate::cli::{validate_slug, Stage};
use crate::config::{save_config, Config, Profile};
use crate::output::{CommandOutput, Output};

/// Run `rw profile <name>` – set the named profile as default. Errors if the profile does not exist.
pub fn set_default(name: &str, config: &mut Config, out: &Output) -> Result<()> {
    if !config.profiles.contains_key(name) {
        anyhow::bail!(
            "profile '{}' does not exist; use 'rw profiles add {}' to add it",
            name,
            name
        );
    }
    config.default = Some(name.to_string());
    save_config(config)?;
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
    save_config(config)?;
    out.print(&AddOutput {
        name: name.to_string(),
        organization,
        stage,
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
            default: None,
            profiles,
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
        let mut config = Config {
            default: None,
            profiles: BTreeMap::new(),
        };
        let err = set_default("missing", &mut config, &out_plain()).unwrap_err();
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
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let err = add(
            "demo",
            Some("mercy".to_string()),
            Some(Stage::Prod),
            &mut config,
            &out_plain(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_create_errors_with_json_flag_when_organization_missing() {
        let mut config = Config {
            default: None,
            profiles: BTreeMap::new(),
        };
        let err = add("demo", None, Some(Stage::Prod), &mut config, &out_json()).unwrap_err();
        assert!(err.to_string().contains("--json"));
    }

    #[test]
    fn test_create_errors_with_json_flag_when_stage_missing() {
        let mut config = Config {
            default: None,
            profiles: BTreeMap::new(),
        };
        let err = add(
            "demo",
            Some("mercy".to_string()),
            None,
            &mut config,
            &out_json(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("--json"));
    }
}
