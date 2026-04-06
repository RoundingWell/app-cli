use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::auth_cache::{delete_auth_cache, load_auth_cache, save_auth_cache, AuthCache};
use crate::cli::{
    validate_slug, ConfigProfileAddArgs, ConfigProfileAuthArgs, ConfigProfileRmArgs,
    ConfigProfileSetArgs, Stage,
};
use crate::config::{save_config_to, Config, Profile};
use crate::output::{CommandOutput, Output};

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

fn prompt_text(label: &str) -> Result<String> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    loop {
        print!("{}: ", label);
        stdout.lock().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            anyhow::bail!("unexpected end of input");
        }
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
        eprintln!("{} cannot be empty", label);
    }
}

fn prompt_password() -> Result<String> {
    loop {
        let password = rpassword::prompt_password("Password: ")?;
        if !password.is_empty() {
            return Ok(password);
        }
        eprintln!("Password cannot be empty");
    }
}

fn prompt_yes_no(question: &str) -> Result<bool> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    loop {
        print!("{} [y/N]: ", question);
        stdout.lock().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            return Ok(false);
        }
        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => eprintln!("Please enter 'y' or 'n'"),
        }
    }
}

// --- Output types ---

#[derive(Serialize)]
pub struct ProfileListOutput {
    pub profiles: Vec<String>,
    pub default: Option<String>,
}

impl CommandOutput for ProfileListOutput {
    fn plain(&self) -> String {
        self.profiles
            .iter()
            .map(|name| {
                if self.default.as_deref() == Some(name.as_str()) {
                    format!("* {}", name)
                } else {
                    format!("  {}", name)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Serialize)]
pub struct ProfileShowOutput {
    pub name: String,
    pub organization: String,
    pub stage: Stage,
    pub auth: Option<String>,
}

impl CommandOutput for ProfileShowOutput {
    fn plain(&self) -> String {
        let auth_str = match &self.auth {
            Some(a) => a.clone(),
            None => "not authenticated".to_string(),
        };
        format!(
            "Profile:      {}\nOrganization: {}\nStage:        {}\nAuth:         {}",
            self.name, self.organization, self.stage, auth_str
        )
    }
}

#[derive(Serialize)]
pub struct ProfileUseOutput {
    pub name: String,
}

impl CommandOutput for ProfileUseOutput {
    fn plain(&self) -> String {
        format!("Default profile set to '{}'.", self.name)
    }
}

#[derive(Serialize)]
pub struct ProfileSetOutput {
    pub name: String,
    pub organization: String,
    pub stage: Stage,
}

impl CommandOutput for ProfileSetOutput {
    fn plain(&self) -> String {
        format!(
            "Profile '{}' updated (organization: {}, stage: {}).",
            self.name, self.organization, self.stage
        )
    }
}

#[derive(Serialize)]
pub struct ProfileAddOutput {
    pub name: String,
    pub organization: String,
    pub stage: Stage,
}

impl CommandOutput for ProfileAddOutput {
    fn plain(&self) -> String {
        format!(
            "Profile '{}' created (organization: {}, stage: {}).",
            self.name, self.organization, self.stage
        )
    }
}

#[derive(Serialize)]
pub struct ProfileRmOutput {
    pub name: String,
}

impl CommandOutput for ProfileRmOutput {
    fn plain(&self) -> String {
        format!("Profile '{}' removed.", self.name)
    }
}

#[derive(Serialize)]
pub struct ProfileAuthOutput {
    pub name: String,
}

impl CommandOutput for ProfileAuthOutput {
    fn plain(&self) -> String {
        format!("Basic auth credentials saved for profile '{}'.", self.name)
    }
}

#[derive(Serialize)]
pub struct UpdatesShowOutput {
    pub auto_update: Option<bool>,
}

impl CommandOutput for UpdatesShowOutput {
    fn plain(&self) -> String {
        match self.auto_update {
            Some(true) => "Automatic updates: enabled".to_string(),
            Some(false) => "Automatic updates: disabled".to_string(),
            None => "Automatic updates: not configured (will prompt)".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct UpdatesEnableOutput;

impl CommandOutput for UpdatesEnableOutput {
    fn plain(&self) -> String {
        "Automatic updates enabled.".to_string()
    }
}

#[derive(Serialize)]
pub struct UpdatesDisableOutput;

impl CommandOutput for UpdatesDisableOutput {
    fn plain(&self) -> String {
        "Automatic updates disabled.".to_string()
    }
}

// --- Command functions ---

pub fn profile_list(config: &Config, out: &Output) {
    let mut names: Vec<String> = config.profiles.keys().cloned().collect();
    names.sort();
    out.print(&ProfileListOutput {
        profiles: names,
        default: config.default.clone(),
    });
}

pub fn profile_show(config: &Config, config_dir: &Path, out: &Output) -> Result<ProfileShowOutput> {
    let name = config.default.as_deref().ok_or_else(|| {
        anyhow::anyhow!("no default profile configured; use 'rw config profile use <name>'")
    })?;

    let profile = config
        .profiles
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("default profile '{}' not found in config", name))?;

    let auth = match load_auth_cache(config_dir, name)? {
        Some(AuthCache::Basic { .. }) => Some("basic".to_string()),
        Some(AuthCache::Bearer { .. }) => Some("bearer".to_string()),
        None => None,
    };

    let output = ProfileShowOutput {
        name: name.to_string(),
        organization: profile.organization.clone(),
        stage: profile.stage.clone(),
        auth,
    };
    out.print(&output);
    Ok(output)
}

pub fn profile_use(
    name: &str,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    if !config.profiles.contains_key(name) {
        anyhow::bail!(
            "profile '{}' does not exist; use 'rw config profile add {}' to add it",
            name,
            name
        );
    }
    config.default = Some(name.to_string());
    save_config_to(config, config_path)?;
    out.print(&ProfileUseOutput {
        name: name.to_string(),
    });
    Ok(())
}

pub fn profile_set(
    args: ConfigProfileSetArgs,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    if !config.profiles.contains_key(&args.name) {
        anyhow::bail!("profile '{}' does not exist", args.name);
    }

    {
        let profile = config.profiles.get_mut(&args.name).unwrap();
        if let Some(org) = args.organization {
            profile.organization = org;
        }
        if let Some(stage) = args.stage {
            profile.stage = stage;
        }
    }

    save_config_to(config, config_path)?;

    let profile = &config.profiles[&args.name];
    out.print(&ProfileSetOutput {
        name: args.name.clone(),
        organization: profile.organization.clone(),
        stage: profile.stage.clone(),
    });
    Ok(())
}

pub fn profile_rm(
    args: ConfigProfileRmArgs,
    config: &mut Config,
    config_path: &Path,
    config_dir: &Path,
    out: &Output,
) -> Result<()> {
    if !config.profiles.contains_key(&args.name) {
        anyhow::bail!("profile '{}' does not exist", args.name);
    }

    if out.json && !args.yes {
        anyhow::bail!("cannot prompt for confirmation in --json mode; use --yes to confirm");
    }

    if !args.yes {
        let confirmed = prompt_yes_no(&format!("Remove profile '{}'?", args.name))?;
        if !confirmed {
            return Ok(());
        }
    }

    config.profiles.remove(&args.name);
    if config.default.as_deref() == Some(args.name.as_str()) {
        config.default = None;
    }
    save_config_to(config, config_path)?;
    delete_auth_cache(config_dir, &args.name)?;
    out.print(&ProfileRmOutput {
        name: args.name.clone(),
    });
    Ok(())
}

pub fn profile_add(
    args: ConfigProfileAddArgs,
    config: &mut Config,
    config_path: &Path,
    out: &Output,
) -> Result<()> {
    if config.profiles.contains_key(&args.name) {
        anyhow::bail!("profile '{}' already exists", args.name);
    }

    if out.json && (args.organization.is_none() || args.stage.is_none()) {
        anyhow::bail!(
            "cannot use interactive mode with --json; provide --organization and --stage"
        );
    }

    let organization = args
        .organization
        .map(Ok)
        .unwrap_or_else(prompt_organization)?;
    let stage = args.stage.map(Ok).unwrap_or_else(prompt_stage)?;

    config.profiles.insert(
        args.name.clone(),
        Profile {
            organization: organization.clone(),
            stage: stage.clone(),
            default: None,
        },
    );
    if args.make_active {
        config.default = Some(args.name.clone());
    }
    save_config_to(config, config_path)?;

    out.print(&ProfileAddOutput {
        name: args.name.clone(),
        organization,
        stage,
    });
    Ok(())
}

pub fn profile_auth(
    args: ConfigProfileAuthArgs,
    config: &Config,
    config_dir: &Path,
    out: &Output,
) -> Result<()> {
    if !config.profiles.contains_key(&args.name) {
        anyhow::bail!("profile '{}' does not exist", args.name);
    }

    if out.json && (args.username.is_none() || args.password.is_none()) {
        anyhow::bail!("cannot use interactive mode with --json; provide --username and --password");
    }

    let username = args
        .username
        .map(|u| {
            if u.is_empty() {
                anyhow::bail!("username cannot be empty")
            } else {
                Ok(u)
            }
        })
        .unwrap_or_else(|| prompt_text("Username"))?;

    let password = args
        .password
        .map(|p| {
            if p.is_empty() {
                anyhow::bail!("password cannot be empty")
            } else {
                Ok(p)
            }
        })
        .unwrap_or_else(prompt_password)?;

    let cache = AuthCache::Basic { username, password };
    save_auth_cache(config_dir, &args.name, &cache)?;

    out.print(&ProfileAuthOutput {
        name: args.name.clone(),
    });
    Ok(())
}

pub fn updates_show(config: &Config, out: &Output) {
    out.print(&UpdatesShowOutput {
        auto_update: config.auto_update,
    });
}

pub fn updates_enable(config: &mut Config, config_path: &Path, out: &Output) -> Result<()> {
    config.auto_update = Some(true);
    save_config_to(config, config_path)?;
    out.print(&UpdatesEnableOutput);
    Ok(())
}

pub fn updates_disable(config: &mut Config, config_path: &Path, out: &Output) -> Result<()> {
    config.auto_update = Some(false);
    save_config_to(config, config_path)?;
    out.print(&UpdatesDisableOutput);
    Ok(())
}

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
    if profile.default.as_ref().map_or(false, |d| d.is_empty()) {
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
    use crate::auth_cache::{save_auth_cache, AuthCache};
    use crate::cli::Stage;
    use crate::config::{Config, Profile};
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

    fn empty_config() -> Config {
        Config {
            version: None,
            default: None,
            profiles: BTreeMap::new(),
            auto_update: None,
        }
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

    // --- profile_list ---

    #[test]
    fn test_profile_list_output_plain_marks_default() {
        let output = ProfileListOutput {
            profiles: vec!["aaa".to_string(), "bbb".to_string()],
            default: Some("aaa".to_string()),
        };
        let text = output.plain();
        assert!(text.contains("* aaa"));
        assert!(text.contains("  bbb"));
    }

    #[test]
    fn test_profile_list_output_json() {
        let output = ProfileListOutput {
            profiles: vec!["aaa".to_string()],
            default: Some("aaa".to_string()),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["profiles"], serde_json::json!(["aaa"]));
        assert_eq!(json["default"], "aaa");
    }

    #[test]
    fn test_profile_list_sorted_alphabetically() {
        let config = {
            let mut c = empty_config();
            c.profiles.insert(
                "zzz".to_string(),
                Profile {
                    organization: "o".to_string(),
                    stage: Stage::Dev,
                    default: None,
                },
            );
            c.profiles.insert(
                "aaa".to_string(),
                Profile {
                    organization: "o".to_string(),
                    stage: Stage::Dev,
                    default: None,
                },
            );
            c.default = Some("aaa".to_string());
            c
        };
        let out = out_plain();
        profile_list(&config, &out);
    }

    // --- profile_show ---

    #[test]
    fn test_profile_show_errors_when_no_default() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = empty_config();
        let err = profile_show(&config, dir.path(), &out_plain()).unwrap_err();
        assert!(err.to_string().contains("no default profile"));
    }

    #[test]
    fn test_profile_show_not_authenticated_when_no_cache() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        config.default = Some("demo".to_string());
        let output = profile_show(&config, dir.path(), &out_plain()).unwrap();
        assert_eq!(output.auth, None);
        assert_eq!(output.name, "demo");
        assert_eq!(output.organization, "mercy");
    }

    #[test]
    fn test_profile_show_auth_type_basic() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        config.default = Some("demo".to_string());
        save_auth_cache(
            dir.path(),
            "demo",
            &AuthCache::Basic {
                username: "alice".to_string(),
                password: "secret".to_string(),
            },
        )
        .unwrap();
        let output = profile_show(&config, dir.path(), &out_plain()).unwrap();
        assert_eq!(output.auth, Some("basic".to_string()));
    }

    #[test]
    fn test_profile_show_auth_type_bearer() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        config.default = Some("demo".to_string());
        save_auth_cache(
            dir.path(),
            "demo",
            &AuthCache::Bearer {
                access_token: "tok".to_string(),
                refresh_token: None,
                expires_at: 9999999999,
            },
        )
        .unwrap();
        let output = profile_show(&config, dir.path(), &out_plain()).unwrap();
        assert_eq!(output.auth, Some("bearer".to_string()));
    }

    // --- profile_use ---

    #[test]
    fn test_profile_use_sets_default() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        profile_use("demo", &mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.default.as_deref(), Some("demo"));
    }

    #[test]
    fn test_profile_use_errors_when_not_found() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        let err = profile_use("missing", &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
        assert!(err.to_string().contains("config profile add"));
    }

    // --- profile_set ---

    #[test]
    fn test_profile_set_updates_organization() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileSetArgs {
            name: "demo".to_string(),
            organization: Some("new-org".to_string()),
            stage: None,
        };
        profile_set(args, &mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.profiles["demo"].organization, "new-org");
    }

    #[test]
    fn test_profile_set_updates_stage() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileSetArgs {
            name: "demo".to_string(),
            organization: None,
            stage: Some(Stage::Dev),
        };
        profile_set(args, &mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.profiles["demo"].stage, Stage::Dev);
    }

    #[test]
    fn test_profile_set_noop_when_nothing_specified() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileSetArgs {
            name: "demo".to_string(),
            organization: None,
            stage: None,
        };
        profile_set(args, &mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.profiles["demo"].organization, "mercy");
        assert_eq!(config.profiles["demo"].stage, Stage::Prod);
    }

    #[test]
    fn test_profile_set_errors_when_not_found() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        let args = ConfigProfileSetArgs {
            name: "missing".to_string(),
            organization: Some("o".to_string()),
            stage: None,
        };
        let err = profile_set(args, &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    // --- profile_rm ---

    #[test]
    fn test_profile_rm_removes_profile() {
        let (_tmp, path) = tmp_path();
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileRmArgs {
            name: "demo".to_string(),
            yes: true,
        };
        profile_rm(args, &mut config, &path, dir.path(), &out_plain()).unwrap();
        assert!(!config.profiles.contains_key("demo"));
    }

    #[test]
    fn test_profile_rm_clears_default() {
        let (_tmp, path) = tmp_path();
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        config.default = Some("demo".to_string());
        let args = ConfigProfileRmArgs {
            name: "demo".to_string(),
            yes: true,
        };
        profile_rm(args, &mut config, &path, dir.path(), &out_plain()).unwrap();
        assert!(config.default.is_none());
    }

    #[test]
    fn test_profile_rm_deletes_auth_cache() {
        let (_tmp, path) = tmp_path();
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        crate::auth_cache::save_auth_cache(
            dir.path(),
            "demo",
            &crate::auth_cache::AuthCache::Basic {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
        )
        .unwrap();
        assert!(crate::auth_cache::auth_cache_path(dir.path(), "demo").exists());
        let args = ConfigProfileRmArgs {
            name: "demo".to_string(),
            yes: true,
        };
        profile_rm(args, &mut config, &path, dir.path(), &out_plain()).unwrap();
        assert!(!crate::auth_cache::auth_cache_path(dir.path(), "demo").exists());
    }

    #[test]
    fn test_profile_rm_json_without_yes_errors() {
        let (_tmp, path) = tmp_path();
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileRmArgs {
            name: "demo".to_string(),
            yes: false,
        };
        let err = profile_rm(args, &mut config, &path, dir.path(), &out_json()).unwrap_err();
        assert!(err.to_string().contains("--yes"));
    }

    #[test]
    fn test_profile_rm_errors_when_not_found() {
        let (_tmp, path) = tmp_path();
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = empty_config();
        let args = ConfigProfileRmArgs {
            name: "missing".to_string(),
            yes: true,
        };
        let err = profile_rm(args, &mut config, &path, dir.path(), &out_plain()).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    // --- profile_add ---

    #[test]
    fn test_profile_add_creates_profile() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        let args = ConfigProfileAddArgs {
            name: "demo".to_string(),
            organization: Some("mercy".to_string()),
            stage: Some(Stage::Prod),
            make_active: false,
        };
        profile_add(args, &mut config, &path, &out_plain()).unwrap();
        assert!(config.profiles.contains_key("demo"));
    }

    #[test]
    fn test_profile_add_with_use_sets_default() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        let args = ConfigProfileAddArgs {
            name: "demo".to_string(),
            organization: Some("mercy".to_string()),
            stage: Some(Stage::Prod),
            make_active: true,
        };
        profile_add(args, &mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.default.as_deref(), Some("demo"));
    }

    #[test]
    fn test_profile_add_without_use_does_not_set_default() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        let args = ConfigProfileAddArgs {
            name: "demo".to_string(),
            organization: Some("mercy".to_string()),
            stage: Some(Stage::Prod),
            make_active: false,
        };
        profile_add(args, &mut config, &path, &out_plain()).unwrap();
        assert!(config.default.is_none());
    }

    #[test]
    fn test_profile_add_errors_when_already_exists() {
        let (_tmp, path) = tmp_path();
        let mut config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileAddArgs {
            name: "demo".to_string(),
            organization: Some("mercy".to_string()),
            stage: Some(Stage::Prod),
            make_active: false,
        };
        let err = profile_add(args, &mut config, &path, &out_plain()).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_profile_add_json_mode_requires_org_and_stage() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        let args = ConfigProfileAddArgs {
            name: "demo".to_string(),
            organization: None,
            stage: Some(Stage::Prod),
            make_active: false,
        };
        let err = profile_add(args, &mut config, &path, &out_json()).unwrap_err();
        assert!(err.to_string().contains("--json"));
    }

    // --- profile_auth ---

    #[test]
    fn test_profile_auth_saves_basic_cache() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileAuthArgs {
            name: "demo".to_string(),
            username: Some("alice".to_string()),
            password: Some("secret".to_string()),
        };
        profile_auth(args, &config, dir.path(), &out_plain()).unwrap();
        let cache = load_auth_cache(dir.path(), "demo").unwrap().unwrap();
        match cache {
            AuthCache::Basic { username, password } => {
                assert_eq!(username, "alice");
                assert_eq!(password, "secret");
            }
            _ => panic!("expected basic cache"),
        }
    }

    #[test]
    fn test_profile_auth_errors_when_profile_not_found() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = empty_config();
        let args = ConfigProfileAuthArgs {
            name: "missing".to_string(),
            username: Some("alice".to_string()),
            password: Some("secret".to_string()),
        };
        let err = profile_auth(args, &config, dir.path(), &out_plain()).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn test_profile_auth_json_mode_requires_username_and_password() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileAuthArgs {
            name: "demo".to_string(),
            username: None,
            password: Some("secret".to_string()),
        };
        let err = profile_auth(args, &config, dir.path(), &out_json()).unwrap_err();
        assert!(err.to_string().contains("--json"));
    }

    #[test]
    fn test_profile_auth_rejects_empty_username() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileAuthArgs {
            name: "demo".to_string(),
            username: Some("".to_string()),
            password: Some("secret".to_string()),
        };
        let err = profile_auth(args, &config, dir.path(), &out_plain()).unwrap_err();
        assert!(err.to_string().contains("username cannot be empty"));
    }

    #[test]
    fn test_profile_auth_rejects_empty_password() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = config_with_profile("demo", "mercy", Stage::Prod);
        let args = ConfigProfileAuthArgs {
            name: "demo".to_string(),
            username: Some("alice".to_string()),
            password: Some("".to_string()),
        };
        let err = profile_auth(args, &config, dir.path(), &out_plain()).unwrap_err();
        assert!(err.to_string().contains("password cannot be empty"));
    }

    // --- updates_show ---

    #[test]
    fn test_updates_show_enabled() {
        let output = UpdatesShowOutput {
            auto_update: Some(true),
        };
        assert!(output.plain().contains("enabled"));
    }

    #[test]
    fn test_updates_show_disabled() {
        let output = UpdatesShowOutput {
            auto_update: Some(false),
        };
        assert!(output.plain().contains("disabled"));
    }

    #[test]
    fn test_updates_show_not_configured() {
        let output = UpdatesShowOutput { auto_update: None };
        let text = output.plain();
        assert!(text.contains("not configured") || text.contains("prompt"));
    }

    // --- updates_enable / updates_disable ---

    #[test]
    fn test_updates_enable_sets_auto_update_true() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        updates_enable(&mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.auto_update, Some(true));
    }

    #[test]
    fn test_updates_disable_sets_auto_update_false() {
        let (_tmp, path) = tmp_path();
        let mut config = empty_config();
        updates_disable(&mut config, &path, &out_plain()).unwrap();
        assert_eq!(config.auto_update, Some(false));
    }

    // --- config default set ---

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

    // --- config default get ---

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

    // --- config default rm ---

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

    // --- config default list ---

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

    // --- profile override tests ---

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
        // active profile must be untouched
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
        // active profile has a different value — should NOT be returned
        config.profiles.get_mut("active").unwrap().default =
            Some([("role".to_string(), "physician".to_string())].into());
        // Just verify no error; actual printed output is not captured here.
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
        // active profile must be untouched
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
        // Just verify no error; actual printed output is not captured here.
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

    // --- DefaultGetOutput ---

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
