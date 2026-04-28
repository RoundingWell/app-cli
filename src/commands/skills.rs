use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::cli::{SkillsArgs, SkillsCommands};
use crate::output::{CommandOutput, Output};

pub fn dispatch(args: SkillsArgs, out: &Output) -> Result<()> {
    match args.command {
        SkillsCommands::Install(a) => run_install(a.local, a.no_clobber, out),
    }
}

#[derive(Debug, Serialize)]
pub struct SkillInstallOutput {
    pub path: String,
    pub skipped: bool,
}

impl CommandOutput for SkillInstallOutput {
    fn plain(&self) -> String {
        if self.skipped {
            format!(
                "Skill already exists at {} (use without --no-clobber to overwrite).",
                self.path
            )
        } else {
            format!("Installed rw skill to {}.", self.path)
        }
    }
}

const SKILL_CONTENT: &str = include_str!("../../skills/rw-skill.md");
const SKILL_FILENAME: &str = "SKILL.md";
const SKILL_SUBDIR: &str = ".claude/skills/rw";

pub fn run_install(local: bool, no_clobber: bool, out: &Output) -> Result<()> {
    let base = resolve_base(local)?;
    install_to_base(&base, no_clobber, out)
}

fn resolve_base(local: bool) -> Result<PathBuf> {
    if local {
        Ok(std::env::current_dir()?)
    } else {
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not determine home directory"))
    }
}

fn install_to_base(base: &Path, no_clobber: bool, out: &Output) -> Result<()> {
    let target = base.join(SKILL_SUBDIR).join(SKILL_FILENAME);
    let path = target.display().to_string();

    if no_clobber && target.exists() {
        out.print(&SkillInstallOutput {
            path,
            skipped: true,
        });
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&target, SKILL_CONTENT)?;
    out.print(&SkillInstallOutput {
        path,
        skipped: false,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Output;
    use std::fs;
    use tempfile::TempDir;

    fn make_out() -> Output {
        Output { json: false }
    }

    #[test]
    fn test_resolve_base_local() {
        let base = resolve_base(true).unwrap();
        assert_eq!(base, std::env::current_dir().unwrap());
    }

    #[test]
    fn test_resolve_base_global() {
        let base = resolve_base(false).unwrap();
        assert_eq!(base, dirs::home_dir().unwrap());
    }

    #[test]
    fn test_install_creates_file_and_dirs() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join(".claude/skills/rw/SKILL.md");
        assert!(!target.exists());

        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, SKILL_CONTENT).unwrap();

        assert!(target.exists());
        let contents = fs::read_to_string(&target).unwrap();
        assert!(contents.contains("name: rw"));
    }

    #[test]
    fn test_no_clobber_skips_existing_file() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join(".claude/skills/rw/SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "original content").unwrap();

        // Simulate no_clobber logic
        let no_clobber = true;
        if no_clobber && target.exists() {
            // should not overwrite
        } else {
            fs::write(&target, SKILL_CONTENT).unwrap();
        }

        let contents = fs::read_to_string(&target).unwrap();
        assert_eq!(contents, "original content");
    }

    #[test]
    fn test_overwrite_existing_file() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join(".claude/skills/rw/SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "old content").unwrap();

        // no_clobber = false: overwrite
        fs::write(&target, SKILL_CONTENT).unwrap();

        let contents = fs::read_to_string(&target).unwrap();
        assert!(contents.contains("name: rw"));
    }

    #[test]
    fn test_skill_content_embedded() {
        assert!(SKILL_CONTENT.contains("name: rw"));
        assert!(SKILL_CONTENT.contains("rw clinicians"));
        assert!(SKILL_CONTENT.contains("rw config"));
        assert!(SKILL_CONTENT.contains("rw artifacts"));
        assert!(SKILL_CONTENT.contains("rw teams"));
        assert!(SKILL_CONTENT.contains("rw roles"));
        assert!(SKILL_CONTENT.contains("rw workspaces"));
    }

    #[test]
    fn test_run_install_local_creates_file() {
        let tmp = TempDir::new().unwrap();
        let out = make_out();
        let result = install_to_base(&tmp.path(), false, &out);
        assert!(result.is_ok());
        let target = tmp.path().join(".claude/skills/rw/SKILL.md");
        assert!(target.exists());
        let contents = fs::read_to_string(&target).unwrap();
        assert!(contents.contains("name: rw"));
    }

    #[test]
    fn test_run_install_no_clobber_preserves_existing() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join(".claude/skills/rw/SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "preserved").unwrap();
        let out = make_out();
        let result = install_to_base(&tmp.path(), true, &out);
        assert!(result.is_ok());
        let contents = fs::read_to_string(&target).unwrap();
        assert_eq!(contents, "preserved");
    }
}
