use anyhow::Result;
use serde::Serialize;

use crate::auth_cache::{save_auth_cache, AuthCache};
use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

#[derive(Serialize)]
struct BasicSetOutput {
    organization: String,
    stage: String,
}

impl CommandOutput for BasicSetOutput {
    fn plain(&self) -> String {
        format!(
            "✓ Basic auth credentials saved for {}/{}.",
            self.organization, self.stage
        )
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

/// Run `rw basic set` – store basic auth credentials for the profile's organization+stage.
///
/// Username and password not supplied via flags are collected interactively. The password
/// is prompted with hidden input. When `--json` is active, both values must be supplied
/// as flags because interactive prompting is not possible.
pub fn set(
    ctx: &AppContext,
    username: Option<String>,
    password: Option<String>,
    out: &Output,
) -> Result<()> {
    if out.json && (username.is_none() || password.is_none()) {
        anyhow::bail!("cannot use interactive mode with --json; provide --username and --password");
    }

    let username = username
        .map(|u| {
            if u.is_empty() {
                anyhow::bail!("username cannot be empty")
            } else {
                Ok(u)
            }
        })
        .unwrap_or_else(|| prompt_text("Username"))?;
    let password = password
        .map(|p| {
            if p.is_empty() {
                anyhow::bail!("password cannot be empty")
            } else {
                Ok(p)
            }
        })
        .unwrap_or_else(prompt_password)?;

    let cache = AuthCache::Basic { username, password };
    save_auth_cache(&ctx.config_dir, &ctx.profile, &cache)?;

    out.print(&BasicSetOutput {
        organization: ctx.organization.clone(),
        stage: ctx.stage.to_string(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_cache::load_auth_cache;
    use crate::cli::Stage;
    use crate::output::Output;

    fn test_ctx(dir: &tempfile::TempDir) -> AppContext {
        AppContext {
            config_dir: dir.path().to_path_buf(),
            profile: "test".to_string(),
            organization: "my-org".to_string(),
            stage: Stage::Dev,
            base_url: String::new(),
        }
    }

    #[test]
    fn test_set_saves_basic_cache() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = test_ctx(&dir);
        let out = Output { json: false };
        set(
            &ctx,
            Some("alice".to_string()),
            Some("secret".to_string()),
            &out,
        )
        .unwrap();

        let cache = load_auth_cache(dir.path(), "test").unwrap().unwrap();
        match cache {
            AuthCache::Basic { username, password } => {
                assert_eq!(username, "alice");
                assert_eq!(password, "secret");
            }
            _ => panic!("expected basic cache"),
        }
    }

    #[test]
    fn test_set_json_mode_requires_username_and_password() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = test_ctx(&dir);
        let out = Output { json: true };

        assert!(set(&ctx, None, Some("pw".into()), &out).is_err());
        assert!(set(&ctx, Some("u".into()), None, &out).is_err());
    }

    #[test]
    fn test_set_rejects_empty_username_flag() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = test_ctx(&dir);
        let out = Output { json: false };
        let err = set(&ctx, Some("".into()), Some("pw".into()), &out).unwrap_err();
        assert!(err.to_string().contains("username cannot be empty"));
    }

    #[test]
    fn test_set_rejects_empty_password_flag() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = test_ctx(&dir);
        let out = Output { json: false };
        let err = set(&ctx, Some("alice".into()), Some("".into()), &out).unwrap_err();
        assert!(err.to_string().contains("password cannot be empty"));
    }
}
