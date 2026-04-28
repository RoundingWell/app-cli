//! Interactive prompt helpers shared by `commands::config` and `version_check`.
//!
//! The `*_with` variants take generic `Read`/`Write` so callers can drive them
//! in tests; the public wrappers default to stdin and stderr.
//!
//! Callers in `commands::config` and `version_check` will migrate to these in
//! a follow-up; `dead_code` is suppressed here meanwhile.

#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};

use anyhow::{bail, Result};

use crate::cli::{validate_slug, Stage};

/// Read a single line from `reader` into `buf` (without the trailing newline).
/// Returns `Ok(false)` on EOF.
fn read_line<R: Read>(reader: &mut BufReader<R>, buf: &mut String) -> Result<bool> {
    buf.clear();
    let n = reader.read_line(buf)?;
    if n == 0 {
        return Ok(false);
    }
    if buf.ends_with('\n') {
        buf.pop();
        if buf.ends_with('\r') {
            buf.pop();
        }
    }
    Ok(true)
}

/// `[y/N]` prompt. Empty / `n` / `no` → `false`; `y` / `yes` → `true`.
/// EOF before any answer returns `Ok(false)`.
pub fn yes_no_with<R: Read, W: Write>(reader: R, mut writer: W, question: &str) -> Result<bool> {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        write!(writer, "{} [y/N]: ", question)?;
        writer.flush()?;
        if !read_line(&mut reader, &mut line)? {
            return Ok(false);
        }
        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => writeln!(writer, "Please enter 'y' or 'n'")?,
        }
    }
}

/// Generic non-empty text prompt.
pub fn text_with<R: Read, W: Write>(reader: R, mut writer: W, label: &str) -> Result<String> {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        write!(writer, "{}: ", label)?;
        writer.flush()?;
        if !read_line(&mut reader, &mut line)? {
            bail!("unexpected end of input");
        }
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
        writeln!(writer, "{} cannot be empty", label)?;
    }
}

/// Prompts for an organization slug, re-prompting until a valid slug is entered.
pub fn organization_with<R: Read, W: Write>(reader: R, mut writer: W) -> Result<String> {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        write!(writer, "Organization slug: ")?;
        writer.flush()?;
        if !read_line(&mut reader, &mut line)? {
            bail!("unexpected end of input");
        }
        match validate_slug(line.trim()) {
            Ok(s) => return Ok(s),
            Err(e) => writeln!(writer, "{}", e)?,
        }
    }
}

/// Prompts for a `Stage`, re-prompting on invalid input.
pub fn stage_with<R: Read, W: Write>(reader: R, mut writer: W) -> Result<Stage> {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        write!(writer, "Stage [prod, sandbox, qa, dev, local]: ")?;
        writer.flush()?;
        if !read_line(&mut reader, &mut line)? {
            bail!("unexpected end of input");
        }
        match line.trim() {
            "prod" => return Ok(Stage::Prod),
            "sandbox" => return Ok(Stage::Sandbox),
            "qa" => return Ok(Stage::Qa),
            "dev" => return Ok(Stage::Dev),
            "local" => return Ok(Stage::Local),
            other => writeln!(
                writer,
                "'{}' is not a valid stage; must be one of: prod, sandbox, qa, dev, local",
                other
            )?,
        }
    }
}

// --- stdin/stderr wrappers ---

/// `[y/N]` prompt against stdin, echoing to stderr.
pub fn yes_no(question: &str) -> Result<bool> {
    yes_no_with(std::io::stdin().lock(), std::io::stderr().lock(), question)
}

/// Non-empty text prompt against stdin, echoing to stderr.
pub fn text(label: &str) -> Result<String> {
    text_with(std::io::stdin().lock(), std::io::stderr().lock(), label)
}

/// Organization slug prompt against stdin, echoing to stderr.
pub fn organization() -> Result<String> {
    organization_with(std::io::stdin().lock(), std::io::stderr().lock())
}

/// `Stage` prompt against stdin, echoing to stderr.
pub fn stage() -> Result<Stage> {
    stage_with(std::io::stdin().lock(), std::io::stderr().lock())
}

/// Reads a password from the terminal without echoing it.
/// Re-prompts on empty input. Backed by the `rpassword` crate.
pub fn password() -> Result<String> {
    loop {
        let pw = rpassword::prompt_password("Password: ")?;
        if !pw.is_empty() {
            return Ok(pw);
        }
        eprintln!("Password cannot be empty");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F, T>(input: &str, f: F) -> (Result<T>, String)
    where
        F: FnOnce(&[u8], &mut Vec<u8>) -> Result<T>,
    {
        let mut output = Vec::new();
        let result = f(input.as_bytes(), &mut output);
        (result, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_yes_no_y_returns_true() {
        let (r, out) = run("y\n", |i, o| yes_no_with(i, o, "Continue?"));
        assert!(r.unwrap());
        assert!(out.contains("Continue? [y/N]:"));
    }

    #[test]
    fn test_yes_no_yes_returns_true() {
        let (r, _) = run("yes\n", |i, o| yes_no_with(i, o, "?"));
        assert!(r.unwrap());
    }

    #[test]
    fn test_yes_no_yes_case_insensitive() {
        let (r, _) = run("YES\n", |i, o| yes_no_with(i, o, "?"));
        assert!(r.unwrap());
    }

    #[test]
    fn test_yes_no_n_returns_false() {
        let (r, _) = run("n\n", |i, o| yes_no_with(i, o, "?"));
        assert!(!r.unwrap());
    }

    #[test]
    fn test_yes_no_empty_returns_false() {
        let (r, _) = run("\n", |i, o| yes_no_with(i, o, "?"));
        assert!(!r.unwrap());
    }

    #[test]
    fn test_yes_no_eof_returns_false() {
        let (r, _) = run("", |i, o| yes_no_with(i, o, "?"));
        assert!(!r.unwrap());
    }

    #[test]
    fn test_yes_no_reprompts_on_invalid() {
        let (r, out) = run("maybe\nyes\n", |i, o| yes_no_with(i, o, "?"));
        assert!(r.unwrap());
        assert!(out.contains("Please enter 'y' or 'n'"));
    }

    #[test]
    fn test_text_returns_trimmed_input() {
        let (r, _) = run("  alice  \n", |i, o| text_with(i, o, "Name"));
        assert_eq!(r.unwrap(), "alice");
    }

    #[test]
    fn test_text_reprompts_on_empty() {
        let (r, out) = run("\nbob\n", |i, o| text_with(i, o, "Name"));
        assert_eq!(r.unwrap(), "bob");
        assert!(out.contains("Name cannot be empty"));
    }

    #[test]
    fn test_text_eof_errors() {
        let (r, _) = run("", |i, o| text_with(i, o, "Name"));
        assert!(r.is_err());
    }

    #[test]
    fn test_organization_validates_slug() {
        let (r, out) = run("Bad Slug!\nmercy\n", |i, o| organization_with(i, o));
        assert_eq!(r.unwrap(), "mercy");
        assert!(out.contains("not a valid slug"));
    }

    #[test]
    fn test_stage_parses_each_variant() {
        for (input, expected) in [
            ("prod\n", Stage::Prod),
            ("sandbox\n", Stage::Sandbox),
            ("qa\n", Stage::Qa),
            ("dev\n", Stage::Dev),
            ("local\n", Stage::Local),
        ] {
            let (r, _) = run(input, |i, o| stage_with(i, o));
            assert_eq!(r.unwrap(), expected);
        }
    }

    #[test]
    fn test_stage_reprompts_on_unknown() {
        let (r, out) = run("staging\nprod\n", |i, o| stage_with(i, o));
        assert_eq!(r.unwrap(), Stage::Prod);
        assert!(out.contains("not a valid stage"));
    }
}
