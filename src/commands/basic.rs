use anyhow::Result;
use serde::Serialize;

use crate::auth_cache::{save_auth_cache, AuthCache};
use crate::cli::Stage;
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
    username: Option<String>,
    password: Option<String>,
    organization: &str,
    stage: &Stage,
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
    save_auth_cache(organization, stage, &cache)?;

    out.print(&BasicSetOutput {
        organization: organization.to_string(),
        stage: stage.to_string(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_cache::{delete_auth_cache, load_auth_cache};
    use crate::output::Output;

    /// Removes the test auth cache on drop to keep the real home dir clean.
    struct BasicCacheGuard(&'static str);
    impl Drop for BasicCacheGuard {
        fn drop(&mut self) {
            let _ = delete_auth_cache(self.0, &Stage::Dev);
        }
    }

    #[test]
    fn test_set_saves_basic_cache() {
        let org = "test-basic-set-saves";
        let _guard = BasicCacheGuard(org);

        let out = Output { json: false };
        set(
            Some("alice".to_string()),
            Some("secret".to_string()),
            org,
            &Stage::Dev,
            &out,
        )
        .unwrap();

        let cache = load_auth_cache(org, &Stage::Dev).unwrap().unwrap();
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
        let out = Output { json: true };

        assert!(set(None, Some("pw".into()), "my-org", &Stage::Dev, &out).is_err());
        assert!(set(Some("u".into()), None, "my-org", &Stage::Dev, &out).is_err());
    }

    #[test]
    fn test_set_rejects_empty_username_flag() {
        let out = Output { json: false };
        let err = set(
            Some("".into()),
            Some("pw".into()),
            "my-org",
            &Stage::Dev,
            &out,
        )
        .unwrap_err();
        assert!(err.to_string().contains("username cannot be empty"));
    }

    #[test]
    fn test_set_rejects_empty_password_flag() {
        let out = Output { json: false };
        let err = set(
            Some("alice".into()),
            Some("".into()),
            "my-org",
            &Stage::Dev,
            &out,
        )
        .unwrap_err();
        assert!(err.to_string().contains("password cannot be empty"));
    }
}
