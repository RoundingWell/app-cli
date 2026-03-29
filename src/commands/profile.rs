use anyhow::Result;
use serde::Serialize;

use crate::cli::{validate_slug, Stage};
use crate::config::{save_config, Config, Profile};
use crate::output::{CommandOutput, Output};

/// Run `rw profile <name>` – create the profile if needed (interactively), then set it as default.
pub fn set_default(name: &str, config: &mut Config, out: &Output) -> Result<()> {
    if !config.profiles.contains_key(name) {
        if out.json {
            anyhow::bail!(
                "profile '{}' does not exist; create it interactively without --json first",
                name
            );
        }
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
            match validate_slug(line.trim()) {
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
                    "'{}' is not a valid stage; must be one of: prod, sandbox, qa, dev, local",
                    other
                ),
            }
        };

        config.profiles.insert(
            name.to_string(),
            Profile {
                organization,
                stage,
            },
        );
    }
    config.default = Some(name.to_string());
    save_config(config)?;
    out.print(&SetDefaultOutput {
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

    #[test]
    fn test_set_default_output_plain() {
        let output = SetDefaultOutput {
            name: "demo".to_string(),
        };
        assert_eq!(output.plain(), "Default profile set to 'demo'.");
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
}
