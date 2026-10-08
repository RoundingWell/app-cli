use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Auth credentials stored per profile in `~/.config/rw/auth/{profile}.json`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AuthCache {
    /// Declared before `Bearer`: a cached token gives it the same `access_token` +
    /// `expires_at` fields, and untagged enums take the first variant that matches.
    ClientCredentials {
        client_id: String,
        client_secret: String,
        /// Access token from the last exchange, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        access_token: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires_at: Option<i64>,
    },
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
    /// Returns true if this is a bearer or client-credentials token that is missing,
    /// expired, or expires within 60 seconds.
    pub fn is_expired(&self) -> bool {
        match self {
            AuthCache::ClientCredentials {
                access_token: Some(_),
                expires_at: Some(expires_at),
                ..
            }
            | AuthCache::Bearer { expires_at, .. } => unix_now() >= expires_at - 60,
            AuthCache::ClientCredentials { .. } => true,
            AuthCache::Basic { .. } => false,
        }
    }
}

/// Returns the path to the auth cache file: `{config_dir}/auth/{profile}.json`.
pub fn auth_cache_path(config_dir: &Path, profile: &str) -> PathBuf {
    config_dir.join("auth").join(format!("{}.json", profile))
}

/// Loads the auth cache for the given profile. Returns `None` if no cache exists.
pub fn load_auth_cache(config_dir: &Path, profile: &str) -> Result<Option<AuthCache>> {
    let path = auth_cache_path(config_dir, profile);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read auth cache: {}", path.display()))?;
    let cache: AuthCache = serde_json::from_str(&contents)
        .with_context(|| format!("could not parse auth cache: {}", path.display()))?;
    Ok(Some(cache))
}

/// Persists the auth cache for the given profile, creating directories as needed.
/// The file is written with mode 0600 (owner read/write only) on Unix.
pub fn save_auth_cache(config_dir: &Path, profile: &str, cache: &AuthCache) -> Result<()> {
    let path = auth_cache_path(config_dir, profile);
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

/// Writes `contents` to `path` atomically, with mode 0600 on Unix.
fn write_private_file(path: &std::path::Path, contents: &str) -> Result<()> {
    write_atomic::write_file(path, contents.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Deletes the auth cache file for the given profile.
/// Returns `true` if a file was removed, `false` if none existed.
pub fn delete_auth_cache(config_dir: &Path, profile: &str) -> Result<bool> {
    let path = auth_cache_path(config_dir, profile);
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
    fn test_auth_cache_path() {
        let path = auth_cache_path(Path::new("/cfg"), "demonstration");
        assert!(path.ends_with("auth/demonstration.json"));
    }

    #[test]
    fn test_auth_cache_path_other() {
        let path = auth_cache_path(Path::new("/cfg"), "mercy");
        assert!(path.ends_with("auth/mercy.json"));
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

    fn client_credentials(token: Option<&str>, expires_at: Option<i64>) -> AuthCache {
        AuthCache::ClientCredentials {
            client_id: "id".to_string(),
            client_secret: "sec".to_string(),
            access_token: token.map(str::to_string),
            expires_at,
        }
    }

    #[test]
    fn test_client_credentials_without_token_is_expired() {
        assert!(client_credentials(None, None).is_expired());
    }

    #[test]
    fn test_client_credentials_fresh_token_not_expired() {
        assert!(!client_credentials(Some("t"), Some(unix_now() + 3600)).is_expired());
    }

    #[test]
    fn test_client_credentials_token_in_grace_period_is_expired() {
        assert!(client_credentials(Some("t"), Some(unix_now() + 30)).is_expired());
    }

    #[test]
    fn test_client_credentials_with_cached_token_roundtrips_as_client_credentials() {
        // Must not be swallowed by `Bearer`, which also has access_token + expires_at.
        let json = serde_json::to_string(&client_credentials(Some("t"), Some(9999999999))).unwrap();
        match serde_json::from_str::<AuthCache>(&json).unwrap() {
            AuthCache::ClientCredentials {
                client_id,
                client_secret,
                access_token,
                expires_at,
            } => {
                assert_eq!(client_id, "id");
                assert_eq!(client_secret, "sec");
                assert_eq!(access_token.as_deref(), Some("t"));
                assert_eq!(expires_at, Some(9999999999));
            }
            other => panic!("expected client credentials, got {:?}", other),
        }
    }

    #[test]
    fn test_client_credentials_without_token_omits_token_fields() {
        let json = serde_json::to_string(&client_credentials(None, None)).unwrap();
        assert!(!json.contains("access_token"));
        assert!(!json.contains("expires_at"));
        assert!(matches!(
            serde_json::from_str::<AuthCache>(&json).unwrap(),
            AuthCache::ClientCredentials { .. }
        ));
    }

    #[test]
    fn test_bearer_json_still_parses_as_bearer() {
        let json = r#"{"access_token":"a","refresh_token":"r","expires_at":9999999999}"#;
        assert!(matches!(
            serde_json::from_str::<AuthCache>(json).unwrap(),
            AuthCache::Bearer { .. }
        ));
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
