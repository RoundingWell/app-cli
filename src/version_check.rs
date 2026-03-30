use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::{header, redirect};
use serde::{Deserialize, Serialize};

use crate::output::Output;

const GITHUB_RELEASES_URL: &str = "https://github.com/RoundingWell/app-cli/releases/latest";
const CACHE_FILE: &str = "version_check.json";
const CACHE_TTL_SECS: u64 = 600; // 10 minutes
const REQUEST_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Serialize, Deserialize)]
struct VersionCache {
    checked_at: u64,
    latest_version: String,
}

/// Checks whether a newer version of `rw` is available and, if so, emits a
/// warning via `out.warn()` (stderr).  The check is cached for 10 minutes
/// in `{config_dir}/version_check.json`.  All errors are silently ignored so
/// that network or filesystem problems never interrupt normal CLI usage.
pub async fn check_and_warn(config_dir: &Path, out: &Output) {
    if let Some(latest) = latest_version(config_dir, GITHUB_RELEASES_URL).await {
        let current = env!("CARGO_PKG_VERSION");
        if is_newer(&latest, current) {
            out.warn(&format!(
                "A new version of rw is available: {} (you have {})",
                latest, current
            ));
        }
    }
}

/// Returns the latest version string, either from cache or from GitHub.
/// Returns `None` on any error.
async fn latest_version(config_dir: &Path, url: &str) -> Option<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    let cache_path = config_dir.join(CACHE_FILE);

    // Use cached result if fresh enough.
    if let Some(cache) = load_cache(&cache_path) {
        if now.saturating_sub(cache.checked_at) < CACHE_TTL_SECS {
            return Some(cache.latest_version);
        }
    }

    // Fetch from GitHub.
    let version = fetch_latest_version(url).await?;
    let cache = VersionCache {
        checked_at: now,
        latest_version: version.clone(),
    };
    save_cache(&cache_path, &cache);
    Some(version)
}

fn load_cache(path: &Path) -> Option<VersionCache> {
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn save_cache(path: &Path, cache: &VersionCache) {
    if let Ok(contents) = serde_json::to_string(cache) {
        let _ = write_atomic::write_file(path, contents.as_bytes());
    }
}

/// Issues a HEAD request to `url` without following redirects and extracts the
/// version tag from the `Location` header (e.g. `.../releases/tag/0.3.1`
/// → `"0.3.1"`).
async fn fetch_latest_version(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .redirect(redirect::Policy::none())
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .ok()?;

    let resp = client.head(url).send().await.ok()?;
    let location = resp.headers().get(header::LOCATION)?.to_str().ok()?;
    // Location is like: /RoundingWell/app-cli/releases/tag/0.3.1
    location
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .map(|s| s.to_string())
}

/// Returns true if `candidate` is strictly newer than `current` by semver
/// major.minor.patch comparison.  Falls back to string inequality so that
/// unexpected tag formats still produce a warning rather than silently passing.
fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(c), Some(cur)) => c > cur,
        _ => candidate != current,
    }
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let mut parts = v.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── is_newer ────────────────────────────────────────────────────────────

    #[test]
    fn test_is_newer_patch() {
        assert!(is_newer("0.3.1", "0.3.0"));
        assert!(!is_newer("0.3.0", "0.3.0"));
        assert!(!is_newer("0.3.0", "0.3.1"));
    }

    #[test]
    fn test_is_newer_minor() {
        assert!(is_newer("0.4.0", "0.3.9"));
        assert!(!is_newer("0.3.9", "0.4.0"));
    }

    #[test]
    fn test_is_newer_major() {
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.9.9", "1.0.0"));
    }

    #[test]
    fn test_is_newer_unparseable_falls_back_to_inequality() {
        assert!(is_newer("nightly-abc", "0.3.0"));
        assert!(!is_newer("0.3.0", "0.3.0"));
    }

    // ── parse_version ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_version("bad"), None);
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
    }

    // ── cache helpers ────────────────────────────────────────────────────────

    #[test]
    fn test_cache_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(CACHE_FILE);
        let cache = VersionCache {
            checked_at: 12345,
            latest_version: "0.4.0".to_string(),
        };
        save_cache(&path, &cache);
        let loaded = load_cache(&path).unwrap();
        assert_eq!(loaded.checked_at, 12345);
        assert_eq!(loaded.latest_version, "0.4.0");
    }

    #[test]
    fn test_load_cache_missing_file_returns_none() {
        let dir = TempDir::new().unwrap();
        assert!(load_cache(&dir.path().join("nonexistent.json")).is_none());
    }

    // ── fetch_latest_version ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_fetch_latest_version_reads_location_header() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("HEAD", "/RoundingWell/app-cli/releases/latest")
            .with_status(302)
            .with_header("Location", "/RoundingWell/app-cli/releases/tag/0.5.0")
            .create_async()
            .await;

        let url = format!("{}/RoundingWell/app-cli/releases/latest", server.url());
        let version = fetch_latest_version(&url).await.unwrap();
        assert_eq!(version, "0.5.0");
    }

    #[tokio::test]
    async fn test_fetch_latest_version_trailing_slash_in_location() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("HEAD", "/RoundingWell/app-cli/releases/latest")
            .with_status(302)
            .with_header("Location", "/RoundingWell/app-cli/releases/tag/0.5.0/")
            .create_async()
            .await;

        let url = format!("{}/RoundingWell/app-cli/releases/latest", server.url());
        let version = fetch_latest_version(&url).await.unwrap();
        assert_eq!(version, "0.5.0");
    }

    #[tokio::test]
    async fn test_fetch_latest_version_no_location_returns_none() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("HEAD", "/RoundingWell/app-cli/releases/latest")
            .with_status(200)
            .create_async()
            .await;

        let url = format!("{}/RoundingWell/app-cli/releases/latest", server.url());
        assert!(fetch_latest_version(&url).await.is_none());
    }

    // ── latest_version (cache integration) ──────────────────────────────────

    #[tokio::test]
    async fn test_latest_version_uses_fresh_cache() {
        let dir = TempDir::new().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cache = VersionCache {
            checked_at: now,
            latest_version: "9.9.9".to_string(),
        };
        save_cache(&dir.path().join(CACHE_FILE), &cache);

        // No server needed — fresh cache should be used directly.
        let version = latest_version(dir.path(), "http://unused").await.unwrap();
        assert_eq!(version, "9.9.9");
    }

    #[tokio::test]
    async fn test_latest_version_refreshes_stale_cache() {
        let dir = TempDir::new().unwrap();
        // Write a cache that is older than the TTL.
        let stale_cache = VersionCache {
            checked_at: 0,
            latest_version: "0.1.0".to_string(),
        };
        save_cache(&dir.path().join(CACHE_FILE), &stale_cache);

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("HEAD", "/RoundingWell/app-cli/releases/latest")
            .with_status(302)
            .with_header("Location", "/RoundingWell/app-cli/releases/tag/1.0.0")
            .create_async()
            .await;

        let url = format!("{}/RoundingWell/app-cli/releases/latest", server.url());
        let version = latest_version(dir.path(), &url).await.unwrap();
        assert_eq!(version, "1.0.0");
    }

    // ── check_and_warn ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_check_and_warn_emits_to_stderr_when_newer() {
        let dir = TempDir::new().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cache = VersionCache {
            checked_at: now,
            latest_version: "99.0.0".to_string(),
        };
        save_cache(&dir.path().join(CACHE_FILE), &cache);

        let out = Output { json: false };
        check_and_warn(dir.path(), &out).await; // must not panic
    }
}
