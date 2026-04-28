//! Interactive prompts used by `config profile` commands. These will be
//! replaced by the shared `crate::prompt` module in a follow-up.

use anyhow::Result;

use crate::cli::{validate_slug, Stage};

pub(super) fn prompt_organization() -> Result<String> {
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

pub(super) fn prompt_stage() -> Result<Stage> {
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

pub(super) fn prompt_text(label: &str) -> Result<String> {
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

pub(super) fn prompt_password() -> Result<String> {
    loop {
        let password = rpassword::prompt_password("Password: ")?;
        if !password.is_empty() {
            return Ok(password);
        }
        eprintln!("Password cannot be empty");
    }
}

pub(super) fn prompt_yes_no(question: &str) -> Result<bool> {
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
