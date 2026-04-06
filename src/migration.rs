use anyhow::{Context, Result};
use std::path::Path;

use crate::config::{config_path, save_config_to, Config};

/// Run all pending migrations against `config`, saving the updated config when done.
/// Should be called early in `main`, after loading config and before building the app context.
pub fn run_migrations(config_dir: &Path, config: &mut Config) -> Result<()> {
    if needs_auth_migration(config) {
        migrate_auth_to_profiles(config_dir, config)?;
    }
    Ok(())
}

fn needs_auth_migration(config: &Config) -> bool {
    match &config.version {
        None => true,
        Some(v) => {
            let parts: Vec<u64> = v.split('.').filter_map(|p| p.parse().ok()).collect();
            match parts.as_slice() {
                [major, minor, _patch] => (*major, *minor) < (0, 3),
                _ => true,
            }
        }
    }
}

fn migrate_auth_to_profiles(config_dir: &Path, config: &mut Config) -> Result<()> {
    let auth_dir = config_dir.join("auth");

    for (profile_name, profile) in &config.profiles {
        let old_path = auth_dir.join(format!("{}-{}.json", profile.organization, profile.stage));
        let new_path = auth_dir.join(format!("{}.json", profile_name));

        if old_path.exists() {
            if new_path.exists() {
                eprintln!(
                    "warning: auth file conflict for profile '{}', skipping rename",
                    profile_name
                );
            } else {
                std::fs::rename(&old_path, &new_path).with_context(|| {
                    format!(
                        "could not rename auth file {} to {}",
                        old_path.display(),
                        new_path.display()
                    )
                })?;
            }
        }
    }

    config.version = Some("0.3.0".to_string());
    save_config_to(config, &config_path(config_dir))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Stage;
    use crate::config::Profile;

    fn make_config(profiles: Vec<(&str, &str, Stage)>) -> Config {
        let mut config = Config::default();
        for (name, org, stage) in profiles {
            config.profiles.insert(
                name.to_string(),
                Profile {
                    organization: org.to_string(),
                    stage,
                    default: None,
                },
            );
        }
        config
    }

    fn write_file(path: &std::path::Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn test_migrate_renames_old_auth_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let auth_dir = dir.path().join("auth");
        std::fs::create_dir_all(&auth_dir).unwrap();

        let old_path = auth_dir.join("mercy-sandbox.json");
        write_file(
            &old_path,
            r#"{"access_token":"tok","expires_at":9999999999}"#,
        );

        let mut config = make_config(vec![("mercy", "mercy", Stage::Sandbox)]);
        run_migrations(dir.path(), &mut config).unwrap();

        let new_path = auth_dir.join("mercy.json");
        assert!(new_path.exists(), "new path should exist");
        assert!(!old_path.exists(), "old path should be gone");
        assert_eq!(config.version.as_deref(), Some("0.3.0"));
    }

    #[test]
    fn test_migrate_idempotent() {
        let dir = tempfile::TempDir::new().unwrap();
        let auth_dir = dir.path().join("auth");
        std::fs::create_dir_all(&auth_dir).unwrap();

        let old_path = auth_dir.join("mercy-sandbox.json");
        write_file(
            &old_path,
            r#"{"access_token":"tok","expires_at":9999999999}"#,
        );

        let mut config = make_config(vec![("mercy", "mercy", Stage::Sandbox)]);
        run_migrations(dir.path(), &mut config).unwrap();
        run_migrations(dir.path(), &mut config).unwrap();

        let new_path = auth_dir.join("mercy.json");
        assert!(new_path.exists());
        assert_eq!(config.version.as_deref(), Some("0.3.0"));
    }

    #[test]
    fn test_migrate_conflict_leaves_both_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let auth_dir = dir.path().join("auth");
        std::fs::create_dir_all(&auth_dir).unwrap();

        let old_path = auth_dir.join("mercy-sandbox.json");
        let new_path = auth_dir.join("mercy.json");
        write_file(
            &old_path,
            r#"{"access_token":"old","expires_at":9999999999}"#,
        );
        write_file(
            &new_path,
            r#"{"access_token":"new","expires_at":9999999999}"#,
        );

        let mut config = make_config(vec![("mercy", "mercy", Stage::Sandbox)]);
        run_migrations(dir.path(), &mut config).unwrap();

        assert!(old_path.exists(), "old file should still exist on conflict");
        assert!(new_path.exists(), "new file should still exist");
    }

    #[test]
    fn test_migrate_sets_version() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = make_config(vec![]);
        run_migrations(dir.path(), &mut config).unwrap();
        assert_eq!(config.version.as_deref(), Some("0.3.0"));
    }

    #[test]
    fn test_migrate_skips_when_no_old_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut config = make_config(vec![("mercy", "mercy", Stage::Sandbox)]);
        run_migrations(dir.path(), &mut config).unwrap();
        assert_eq!(config.version.as_deref(), Some("0.3.0"));
    }

    #[test]
    fn test_no_migration_when_version_is_030() {
        let dir = tempfile::TempDir::new().unwrap();
        let auth_dir = dir.path().join("auth");
        std::fs::create_dir_all(&auth_dir).unwrap();

        let old_path = auth_dir.join("mercy-sandbox.json");
        write_file(
            &old_path,
            r#"{"access_token":"tok","expires_at":9999999999}"#,
        );

        let mut config = make_config(vec![("mercy", "mercy", Stage::Sandbox)]);
        config.version = Some("0.3.0".to_string());
        run_migrations(dir.path(), &mut config).unwrap();

        assert!(
            old_path.exists(),
            "old file should be untouched when already migrated"
        );
    }
}
