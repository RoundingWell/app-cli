use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::Stage;

/// Auth credentials stored per organization+stage in `~/.config/rw/auth/{organization}-{stage}.json`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AuthCache {
    Bearer {
        access_token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        refresh_token: Option<String>,
        /// Unix timestamp (seconds) at which the access token expires.
        expires_at: i64,
    },
    Basic {
        username: String,
        password: String,
    },
}

impl AuthCache {
    /// Returns true if this is a bearer token that is expired or expires within 60 seconds.
    pub fn is_expired(&self) -> bool {
        match self {
            AuthCache::Bearer { expires_at, .. } => unix_now() >= expires_at - 60,
            AuthCache::Basic { .. } => false,
        }
    }
}

/// Returns the path to the auth cache file: `~/.config/rw/auth/{organization}-{stage}.json`.
pub fn auth_cache_path(organization: &str, stage: &Stage) -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home
        .join(".config")
        .join("rw")
        .join("auth")
        .join(format!("{}-{}.json", organization, stage)))
}

/// Loads the auth cache for the given organization+stage. Returns `None` if no cache exists.
pub fn load_auth_cache(organization: &str, stage: &Stage) -> Result<Option<AuthCache>> {
    let path = auth_cache_path(organization, stage)?;
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read auth cache: {}", path.display()))?;
    let cache: AuthCache = serde_json::from_str(&contents)
        .with_context(|| format!("could not parse auth cache: {}", path.display()))?;
    Ok(Some(cache))
}

/// Persists the auth cache for the given organization+stage, creating directories as needed.
/// The file is written with mode 0600 (owner read/write only) on Unix.
pub fn save_auth_cache(organization: &str, stage: &Stage, cache: &AuthCache) -> Result<()> {
    let path = auth_cache_path(organization, stage)?;
    if let Some(parent) = path.parent() {
        create_private_dir(parent)
            .with_context(|| format!("could not create auth directory: {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(cache).context("could not serialize auth cache")?;
    write_private_file(&path, &contents)
        .with_context(|| format!("could not write auth cache: {}", path.display()))?;
    Ok(())
}

/// Creates a directory (and parents) with mode 0700 on Unix.
fn create_private_dir(path: &std::path::Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
    }
    #[cfg(not(unix))]
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Writes `contents` to `path` with mode 0600 on Unix.
fn write_private_file(path: &std::path::Path, contents: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?
            .write_all(contents.as_bytes())?;
    }
    #[cfg(not(unix))]
    std::fs::write(path, contents)?;
    Ok(())
}

/// Deletes the auth cache file for the given organization+stage.
/// Returns `true` if a file was removed, `false` if none existed.
pub fn delete_auth_cache(organization: &str, stage: &Stage) -> Result<bool> {
    let path = auth_cache_path(organization, stage)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("could not remove auth cache: {}", path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Computes the absolute expiry timestamp from an `expires_in` duration (seconds).
pub fn expires_at_from_duration(expires_in: u64) -> i64 {
    unix_now() + expires_in as i64
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_cache_path_prod() {
        let path = auth_cache_path("demonstration", &Stage::Prod).unwrap();
        assert!(path.ends_with(".config/rw/auth/demonstration-prod.json"));
    }

    #[test]
    fn test_auth_cache_path_sandbox() {
        let path = auth_cache_path("mercy", &Stage::Sandbox).unwrap();
        assert!(path.ends_with(".config/rw/auth/mercy-sandbox.json"));
    }

    #[test]
    fn test_bearer_not_expired() {
        let cache = AuthCache::Bearer {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: unix_now() + 3600,
        };
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_bearer_expired() {
        let cache = AuthCache::Bearer {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: unix_now() - 1,
        };
        assert!(cache.is_expired());
    }

    #[test]
    fn test_bearer_expires_within_grace_period() {
        let cache = AuthCache::Bearer {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: unix_now() + 30, // expires in 30s, inside the 60s grace period
        };
        assert!(cache.is_expired());
    }

    #[test]
    fn test_basic_never_expired() {
        let cache = AuthCache::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_bearer_serialization_roundtrip() {
        let cache = AuthCache::Bearer {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: 9999999999,
        };
        let json = serde_json::to_string_pretty(&cache).unwrap();
        let loaded: AuthCache = serde_json::from_str(&json).unwrap();
        match loaded {
            AuthCache::Bearer {
                access_token,
                refresh_token,
                expires_at,
            } => {
                assert_eq!(access_token, "access");
                assert_eq!(refresh_token, Some("refresh".to_string()));
                assert_eq!(expires_at, 9999999999);
            }
            _ => panic!("expected bearer"),
        }
    }

    #[test]
    fn test_bearer_without_refresh_token_serialization() {
        let cache = AuthCache::Bearer {
            access_token: "access".to_string(),
            refresh_token: None,
            expires_at: 9999999999,
        };
        let json = serde_json::to_string(&cache).unwrap();
        // refresh_token field should be absent when None
        assert!(!json.contains("refresh_token"));
    }

    #[test]
    fn test_basic_serialization_roundtrip() {
        let cache = AuthCache::Basic {
            username: "alice".to_string(),
            password: "secret".to_string(),
        };
        let json = serde_json::to_string_pretty(&cache).unwrap();
        let loaded: AuthCache = serde_json::from_str(&json).unwrap();
        match loaded {
            AuthCache::Basic { username, password } => {
                assert_eq!(username, "alice");
                assert_eq!(password, "secret");
            }
            _ => panic!("expected basic"),
        }
    }

    #[test]
    fn test_expires_at_from_duration() {
        let before = unix_now();
        let expires_at = expires_at_from_duration(3600);
        let after = unix_now();
        assert!(expires_at >= before + 3600);
        assert!(expires_at <= after + 3600);
    }
}
