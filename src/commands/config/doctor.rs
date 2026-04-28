//! `rw config doctor` — diagnose the active profile's setup.
//!
//! Runs a fixed sequence of checks (profile, auth, API, defaults) and
//! reports each outcome. Returns a non-zero exit when any check fails.
//!
//! Uses `reqwest::Client` directly rather than `ApiClient` because doctor
//! must run with broken/missing config, and must surface non-2xx responses
//! (e.g. 401) as a check failure with status + latency rather than bailing.

use anyhow::{anyhow, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use crate::api::resolve_api;
use crate::auth_cache::{load_auth_cache, AuthCache};
use crate::config::Config;
use crate::output::{CommandOutput, Output};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
    Info,
}

impl CheckStatus {
    fn glyph(self) -> &'static str {
        match self {
            CheckStatus::Pass => "✓",
            CheckStatus::Warn => "⚠",
            CheckStatus::Fail => "✗",
            CheckStatus::Skip => "-",
            CheckStatus::Info => "ℹ",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DoctorOutput {
    pub ok: bool,
    pub checks: Vec<CheckResult>,
}

impl CommandOutput for DoctorOutput {
    fn plain(&self) -> String {
        self.checks
            .iter()
            .map(|c| format!("{} {:<9} {}", c.status.glyph(), c.name, c.message))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Public entry point for `rw config doctor`.
pub async fn doctor(
    config: &Config,
    config_dir: &Path,
    profile_override: Option<&str>,
    out: &Output,
) -> Result<()> {
    let report = run_checks(config, config_dir, profile_override).await;
    out.print(&report);
    if !report.ok {
        return Err(anyhow!("doctor checks failed"));
    }
    Ok(())
}

/// Run all checks and assemble the report. Pure-ish: side effects are limited
/// to filesystem reads (auth cache) and one HTTP request.
pub(crate) async fn run_checks(
    config: &Config,
    config_dir: &Path,
    profile_override: Option<&str>,
) -> DoctorOutput {
    let mut checks: Vec<CheckResult> = Vec::with_capacity(4);

    // 1. Profile.
    let profile = check_profile(config, profile_override);
    let profile_ok = profile.status == CheckStatus::Pass;
    let profile_ctx = if profile_ok {
        resolve_profile_ctx(config, profile_override)
    } else {
        None
    };
    checks.push(profile);

    // Load the auth cache once and share it across the auth and api checks.
    let cache_load: Option<Result<Option<AuthCache>, anyhow::Error>> = profile_ctx
        .as_ref()
        .map(|c| load_auth_cache(config_dir, &c.profile));

    // 2. Auth.
    let auth = match cache_load.as_ref() {
        Some(loaded) => check_auth(loaded),
        None => skip("auth", "profile check failed"),
    };
    let auth_ok = matches!(auth.status, CheckStatus::Pass | CheckStatus::Warn);
    checks.push(auth);

    // 3. API reachability.
    let api = match (&profile_ctx, auth_ok, cache_load.as_ref()) {
        (Some(ctx), true, Some(Ok(Some(cache)))) => check_api(ctx, cache).await,
        _ => skip("api", "auth check failed"),
    };
    checks.push(api);

    // 4. Defaults (informational only).
    let defaults = match &profile_ctx {
        Some(ctx) => check_defaults(config, &ctx.profile),
        None => skip("defaults", "profile check failed"),
    };
    checks.push(defaults);

    let ok = !checks.iter().any(|c| c.status == CheckStatus::Fail);
    DoctorOutput { ok, checks }
}

// --- Internal helpers ---

struct ProfileCtx {
    profile: String,
    organization: String,
    base_url: String,
}

fn resolve_profile_ctx(config: &Config, profile_override: Option<&str>) -> Option<ProfileCtx> {
    let (profile, organization, stage) =
        crate::config::resolve_profile(config, profile_override).ok()?;
    let base_url = resolve_api(&organization, &stage);
    Some(ProfileCtx {
        profile,
        organization,
        base_url,
    })
}

fn skip(name: &str, reason: &str) -> CheckResult {
    CheckResult {
        name: name.to_string(),
        status: CheckStatus::Skip,
        message: format!("skipped ({})", reason),
        details: BTreeMap::new(),
    }
}

fn check_profile(config: &Config, profile_override: Option<&str>) -> CheckResult {
    match crate::config::resolve_profile(config, profile_override) {
        Ok((name, organization, stage)) => {
            let mut details = BTreeMap::new();
            details.insert("profile".to_string(), serde_json::json!(name));
            details.insert("organization".to_string(), serde_json::json!(organization));
            details.insert("stage".to_string(), serde_json::json!(stage.to_string()));
            CheckResult {
                name: "profile".to_string(),
                status: CheckStatus::Pass,
                message: format!(
                    "{} (organization: {}, stage: {})",
                    name, organization, stage
                ),
                details,
            }
        }
        Err(e) => CheckResult {
            name: "profile".to_string(),
            status: CheckStatus::Fail,
            message: format!("{:#}", e),
            details: BTreeMap::new(),
        },
    }
}

fn check_auth(loaded: &Result<Option<AuthCache>, anyhow::Error>) -> CheckResult {
    let cache = match loaded {
        Ok(Some(c)) => c,
        Ok(None) => {
            return CheckResult {
                name: "auth".to_string(),
                status: CheckStatus::Fail,
                message: "no credentials stored — run `rw auth login`".to_string(),
                details: BTreeMap::new(),
            }
        }
        Err(e) => {
            return CheckResult {
                name: "auth".to_string(),
                status: CheckStatus::Fail,
                message: format!("could not read auth cache: {:#}", e),
                details: BTreeMap::new(),
            }
        }
    };

    match cache {
        AuthCache::Bearer {
            refresh_token,
            expires_at,
            ..
        } => {
            let mut details = BTreeMap::new();
            details.insert("type".to_string(), serde_json::json!("bearer"));
            details.insert("expires_at".to_string(), serde_json::json!(expires_at));

            if !cache.is_expired() {
                CheckResult {
                    name: "auth".to_string(),
                    status: CheckStatus::Pass,
                    message: format!("bearer token, valid for {}", remaining(*expires_at)),
                    details,
                }
            } else if refresh_token.is_some() {
                CheckResult {
                    name: "auth".to_string(),
                    status: CheckStatus::Warn,
                    message: "bearer token expired (refresh token available)".to_string(),
                    details,
                }
            } else {
                CheckResult {
                    name: "auth".to_string(),
                    status: CheckStatus::Fail,
                    message: "bearer token expired and no refresh token — run `rw auth login`"
                        .to_string(),
                    details,
                }
            }
        }
        AuthCache::Basic { username, .. } => {
            let mut details = BTreeMap::new();
            details.insert("type".to_string(), serde_json::json!("basic"));
            details.insert("username".to_string(), serde_json::json!(username));
            CheckResult {
                name: "auth".to_string(),
                status: CheckStatus::Pass,
                message: format!("basic auth (user: {})", username),
                details,
            }
        }
    }
}

async fn check_api(ctx: &ProfileCtx, auth: &AuthCache) -> CheckResult {
    let url = format!("{}/clinicians/me", ctx.base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    req = match auth {
        AuthCache::Bearer { access_token, .. } => req.header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        ),
        AuthCache::Basic { username, password } => req.basic_auth(username, Some(password)),
    };

    let started = Instant::now();
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            return CheckResult {
                name: "api".to_string(),
                status: CheckStatus::Fail,
                message: format!("could not reach {}: {}", ctx.base_url, e),
                details: BTreeMap::new(),
            }
        }
    };
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let status = resp.status();

    let mut details = BTreeMap::new();
    details.insert("url".to_string(), serde_json::json!(url));
    details.insert("status".to_string(), serde_json::json!(status.as_u16()));
    details.insert("latency_ms".to_string(), serde_json::json!(elapsed_ms));
    details.insert(
        "organization".to_string(),
        serde_json::json!(ctx.organization),
    );

    if status.is_success() {
        CheckResult {
            name: "api".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "GET /clinicians/me → {} ({}ms)",
                status.as_u16(),
                elapsed_ms
            ),
            details,
        }
    } else {
        CheckResult {
            name: "api".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "GET /clinicians/me → {} ({}ms)",
                status.as_u16(),
                elapsed_ms
            ),
            details,
        }
    }
}

fn check_defaults(config: &Config, profile: &str) -> CheckResult {
    let defaults = config
        .profiles
        .get(profile)
        .and_then(|p| p.default.clone())
        .unwrap_or_default();

    if defaults.is_empty() {
        return CheckResult {
            name: "defaults".to_string(),
            status: CheckStatus::Info,
            message: "no defaults configured".to_string(),
            details: BTreeMap::new(),
        };
    }

    let mut details = BTreeMap::new();
    for (k, v) in &defaults {
        details.insert(k.clone(), serde_json::json!(v));
    }
    let summary = defaults
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(", ");
    CheckResult {
        name: "defaults".to_string(),
        status: CheckStatus::Info,
        message: summary,
        details,
    }
}

/// Renders a remaining duration as a short, human-readable string.
fn remaining(expires_at: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs = (expires_at - now).max(0);
    if secs >= 3600 {
        format!("{}h{:02}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Stage;
    use crate::config::Profile;
    use mockito::Server;

    fn cfg_with_default(stage: Stage) -> Config {
        let mut config = Config::default();
        config.profiles.insert(
            "demo".to_string(),
            Profile {
                organization: "demonstration".to_string(),
                stage,
                default: None,
            },
        );
        config.default = Some("demo".to_string());
        config
    }

    fn unix_now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    // --- check_profile ---

    #[test]
    fn test_check_profile_pass() {
        let config = cfg_with_default(Stage::Prod);
        let r = check_profile(&config, None);
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("demo"));
        assert!(r.message.contains("demonstration"));
        assert!(r.message.contains("prod"));
    }

    #[test]
    fn test_check_profile_fails_without_default() {
        let config = Config::default();
        let r = check_profile(&config, None);
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("no profile selected"));
    }

    #[test]
    fn test_check_profile_fails_for_unknown_override() {
        let config = cfg_with_default(Stage::Prod);
        let r = check_profile(&config, Some("nope"));
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("nope"));
    }

    // --- check_auth ---

    #[test]
    fn test_check_auth_no_cache_fails() {
        let r = check_auth(&Ok(None));
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("no credentials"));
    }

    #[test]
    fn test_check_auth_load_error_fails() {
        let r = check_auth(&Err(anyhow!("boom")));
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("could not read"));
    }

    #[test]
    fn test_check_auth_basic_passes() {
        let r = check_auth(&Ok(Some(AuthCache::Basic {
            username: "alice".to_string(),
            password: "secret".to_string(),
        })));
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("basic"));
        assert!(r.message.contains("alice"));
    }

    #[test]
    fn test_check_auth_bearer_valid_passes() {
        let r = check_auth(&Ok(Some(AuthCache::Bearer {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: unix_now() + 3600,
        })));
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("bearer"));
    }

    #[test]
    fn test_check_auth_bearer_expired_no_refresh_fails() {
        let r = check_auth(&Ok(Some(AuthCache::Bearer {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: unix_now() - 1,
        })));
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("expired"));
    }

    #[test]
    fn test_check_auth_bearer_expired_with_refresh_warns() {
        let r = check_auth(&Ok(Some(AuthCache::Bearer {
            access_token: "tok".to_string(),
            refresh_token: Some("rt".to_string()),
            expires_at: unix_now() - 1,
        })));
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("refresh"));
    }

    #[test]
    fn test_check_auth_bearer_details_omit_token() {
        let r = check_auth(&Ok(Some(AuthCache::Bearer {
            access_token: "supersecret".to_string(),
            refresh_token: Some("rt".to_string()),
            expires_at: unix_now() + 3600,
        })));
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains("supersecret"));
        assert!(!json.contains("\"rt\""));
    }

    // --- check_defaults ---

    #[test]
    fn test_check_defaults_empty_is_info() {
        let config = cfg_with_default(Stage::Prod);
        let r = check_defaults(&config, "demo");
        assert_eq!(r.status, CheckStatus::Info);
        assert!(r.message.contains("no defaults"));
    }

    #[test]
    fn test_check_defaults_with_values() {
        let mut config = cfg_with_default(Stage::Prod);
        let mut defaults = BTreeMap::new();
        defaults.insert("team".to_string(), "NUR".to_string());
        defaults.insert("role".to_string(), "employee".to_string());
        config
            .profiles
            .get_mut("demo")
            .unwrap()
            .default
            .replace(defaults);
        let r = check_defaults(&config, "demo");
        assert_eq!(r.status, CheckStatus::Info);
        assert!(r.message.contains("team=NUR"));
        assert!(r.message.contains("role=employee"));
    }

    // --- check_api (via mockito) ---

    #[tokio::test]
    async fn test_check_api_pass_on_2xx() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/clinicians/me")
            .match_header("authorization", "Bearer t")
            .with_status(200)
            .with_body(r#"{"data":{}}"#)
            .create_async()
            .await;

        let ctx = ProfileCtx {
            profile: "demo".to_string(),
            organization: "demonstration".to_string(),
            base_url: server.url(),
        };
        let auth = AuthCache::Bearer {
            access_token: "t".to_string(),
            refresh_token: None,
            expires_at: unix_now() + 3600,
        };
        let r = check_api(&ctx, &auth).await;
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("200"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_check_api_fail_on_401() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/clinicians/me")
            .with_status(401)
            .with_body("unauthorized")
            .create_async()
            .await;

        let ctx = ProfileCtx {
            profile: "demo".to_string(),
            organization: "demonstration".to_string(),
            base_url: server.url(),
        };
        let auth = AuthCache::Bearer {
            access_token: "t".to_string(),
            refresh_token: None,
            expires_at: unix_now() + 3600,
        };
        let r = check_api(&ctx, &auth).await;
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("401"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_check_api_basic_auth_attached() {
        let mut server = Server::new_async().await;
        // base64("alice:secret") = "YWxpY2U6c2VjcmV0"
        let mock = server
            .mock("GET", "/clinicians/me")
            .match_header("authorization", "Basic YWxpY2U6c2VjcmV0")
            .with_status(200)
            .with_body(r#"{"data":{}}"#)
            .create_async()
            .await;

        let ctx = ProfileCtx {
            profile: "demo".to_string(),
            organization: "demonstration".to_string(),
            base_url: server.url(),
        };
        let auth = AuthCache::Basic {
            username: "alice".to_string(),
            password: "secret".to_string(),
        };
        let r = check_api(&ctx, &auth).await;
        assert_eq!(r.status, CheckStatus::Pass);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_check_api_network_error_fails() {
        // mockito mocks HTTP responses but can't simulate a refused
        // connection. Reserve an ephemeral loopback port via the OS, then
        // drop the listener so any subsequent connect to that address is
        // refused — hermetic (no external network) and deterministic.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let ctx = ProfileCtx {
            profile: "demo".to_string(),
            organization: "demonstration".to_string(),
            base_url: format!("http://{}", addr),
        };
        let auth = AuthCache::Bearer {
            access_token: "t".to_string(),
            refresh_token: None,
            expires_at: unix_now() + 3600,
        };
        let r = check_api(&ctx, &auth).await;
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("could not reach"));
    }

    // --- run_checks integration ---

    #[tokio::test]
    async fn test_run_checks_no_profile_cascades_skips() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = Config::default();
        let report = run_checks(&config, dir.path(), None).await;
        assert!(!report.ok);
        assert_eq!(report.checks.len(), 4);
        assert_eq!(report.checks[0].status, CheckStatus::Fail);
        assert_eq!(report.checks[1].status, CheckStatus::Skip);
        assert_eq!(report.checks[2].status, CheckStatus::Skip);
        assert_eq!(report.checks[3].status, CheckStatus::Skip);
    }

    #[tokio::test]
    async fn test_run_checks_no_auth_skips_api_but_runs_defaults() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = cfg_with_default(Stage::Prod);
        let report = run_checks(&config, dir.path(), None).await;
        assert!(!report.ok);
        assert_eq!(report.checks[0].status, CheckStatus::Pass);
        assert_eq!(report.checks[1].status, CheckStatus::Fail);
        assert_eq!(report.checks[2].status, CheckStatus::Skip);
        // defaults runs because the profile resolved
        assert_eq!(report.checks[3].status, CheckStatus::Info);
    }

    #[tokio::test]
    async fn test_doctor_output_plain_format() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = Config::default();
        let report = run_checks(&config, dir.path(), None).await;
        let plain = report.plain();
        // Each check renders one line with a glyph + name.
        assert_eq!(plain.lines().count(), 4);
        assert!(plain.contains("✗ profile"));
        assert!(plain.contains("- auth"));
        assert!(plain.contains("- api"));
        assert!(plain.contains("- defaults"));
    }

    #[tokio::test]
    async fn test_doctor_output_json_shape() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = Config::default();
        let report = run_checks(&config, dir.path(), None).await;
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["ok"], false);
        assert!(json["checks"].is_array());
        assert_eq!(json["checks"][0]["name"], "profile");
        assert_eq!(json["checks"][0]["status"], "fail");
    }

    #[tokio::test]
    async fn test_doctor_returns_err_when_any_check_fails() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = Config::default();
        let out = Output { json: true };
        let result = doctor(&config, dir.path(), None, &out).await;
        assert!(result.is_err());
    }
}
