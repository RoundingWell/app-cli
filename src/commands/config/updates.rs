//! `rw config updates` subcommands: show / enable / disable.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::config::{save_config_to, Config};
use crate::output::{CommandOutput, Output};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::collections::BTreeMap;

    fn out_plain() -> Output {
        Output { json: false }
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
}
