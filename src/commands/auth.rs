use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::auth_cache::{
    delete_auth_cache, expires_at_from_duration, load_auth_cache, save_auth_cache, AuthCache,
};
use crate::cli::{AuthArgs, AuthCommands, Stage};
use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

pub async fn dispatch(args: AuthArgs, ctx: &AppContext, out: &Output) -> Result<()> {
    match args.command {
        AuthCommands::Login => login(ctx, out).await,
        AuthCommands::Status => status(ctx, out),
        AuthCommands::Header => header(ctx, out).await,
        AuthCommands::Logout => logout(ctx, out),
    }
}

struct WorkOsConfig {
    client_id: &'static str,
    device_auth_url: &'static str,
    token_url: &'static str,
}

const PRODUCTION: WorkOsConfig = WorkOsConfig {
    client_id: "client_01KMREY0MMNCB4B9AK4X9C0TBG",
    device_auth_url: "https://authkit.roundingwell.com/oauth2/device_authorization",
    token_url: "https://authkit.roundingwell.com/oauth2/token",
};

const DEV: WorkOsConfig = WorkOsConfig {
    client_id: "client_01KMRQT9V7YE17BYA5NDSPK572",
    device_auth_url: "https://expansive-market-28-staging.authkit.app/oauth2/device_authorization",
    token_url: "https://expansive-market-28-staging.authkit.app/oauth2/token",
};

fn workos_config(stage: &Stage) -> &'static WorkOsConfig {
    match stage {
        Stage::Prod | Stage::Sandbox => &PRODUCTION,
        _ => &DEV,
    }
}

/// Response from the device authorization endpoint.
#[derive(Debug, serde::Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u64,
    #[serde(default)]
    interval: u64,
}

/// Response from the token endpoint on success.
#[derive(Debug, serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

/// Error response from the token endpoint while polling.
#[derive(Debug, serde::Deserialize)]
struct TokenErrorResponse {
    error: String,
}

// --- Output types ---

#[derive(Serialize)]
pub struct MessageOutput {
    pub message: String,
}

impl CommandOutput for MessageOutput {
    fn plain(&self) -> String {
        self.message.clone()
    }
}

#[derive(Serialize)]
pub struct StatusOutput {
    #[serde(rename = "type")]
    pub auth_type: Option<String>,
    pub authenticated: bool,
    pub expired: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip)]
    pub profile: String,
}

impl CommandOutput for StatusOutput {
    fn plain(&self) -> String {
        match (self.authenticated, self.auth_type.as_deref(), self.expired) {
            (false, _, _) => format!(
                "✗ Not authenticated for profile '{}'.\n  Run `rw auth login` to authenticate.",
                self.profile
            ),
            (true, Some("bearer"), true) => format!(
                "✓ Authenticated using profile '{}' (bearer token, expired – will refresh on next use).",
                self.profile
            ),
            (true, Some("bearer"), false) => format!(
                "✓ Authenticated using profile '{}' (bearer token).",
                self.profile
            ),
            (true, Some("basic"), _) => {
                if let Some(ref u) = self.username {
                    format!(
                        "✓ Authenticated using profile '{}' (basic, user: {}).",
                        self.profile, u
                    )
                } else {
                    format!("✓ Authenticated using profile '{}' (basic).", self.profile)
                }
            }
            _ => format!("✓ Authenticated using profile '{}'.", self.profile),
        }
    }
}

#[derive(Serialize)]
pub struct HeaderOutput {
    pub header: String,
}

impl CommandOutput for HeaderOutput {
    fn plain(&self) -> String {
        self.header.clone()
    }
}

// --- Command implementations ---

/// Run `rw auth login` – use the OAuth Device Authorization Flow to authenticate
/// via WorkOS AuthKit, poll for a token, and persist credentials.
pub async fn login(ctx: &AppContext, out: &Output) -> Result<()> {
    if out.json {
        anyhow::bail!("`rw auth login` is interactive and cannot be used with --json");
    }

    let wos = workos_config(&ctx.stage);
    let client = reqwest::Client::new();

    // Step 1: Request device authorization.
    // `offline_access` scope is required to receive a refresh token in the token response.
    let resp = client
        .post(wos.device_auth_url)
        .form(&[("client_id", wos.client_id), ("scope", "offline_access")])
        .send()
        .await
        .context("failed to reach WorkOS device authorization endpoint")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!(
            "device authorization endpoint returned {}: {}",
            status,
            body
        );
    }

    let device_auth: DeviceAuthResponse = resp
        .json()
        .await
        .context("failed to parse device authorization response")?;

    // Step 2: Open browser for authentication.
    out.info(&format!(
        "Open the following URL in your browser and enter code: {}\n  {}\n\nOr visit this URL to authenticate automatically:\n  {}",
        device_auth.user_code, device_auth.verification_uri, device_auth.verification_uri_complete
    ));

    if let Err(e) = webbrowser::open(&device_auth.verification_uri_complete) {
        out.warn(&format!(
            "Warning: could not open browser automatically: {}",
            e
        ));
    }

    // Step 3: Poll for the token.
    let mut interval_secs = device_auth.interval.max(5);
    let deadline = std::time::Instant::now() + Duration::from_secs(device_auth.expires_in);

    out.info("Waiting for authentication...");

    loop {
        sleep(Duration::from_secs(interval_secs)).await;

        if std::time::Instant::now() >= deadline {
            bail!("authentication timed out – please run `rw auth login` again");
        }

        let poll_resp = client
            .post(wos.token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_auth.device_code),
                ("client_id", wos.client_id),
            ])
            .send()
            .await
            .context("failed to reach WorkOS token endpoint")?;

        let poll_status = poll_resp.status();

        if poll_status.is_success() {
            let token: TokenResponse = poll_resp
                .json()
                .await
                .context("failed to parse token response")?;

            let cache = AuthCache::Bearer {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_at: expires_at_from_duration(token.expires_in),
            };
            save_auth_cache(&ctx.config_dir, &ctx.profile, &cache)?;

            out.print(&MessageOutput {
                message: format!(
                    "✓ Authenticated successfully. Credentials saved for profile '{}'.",
                    ctx.profile
                ),
            });
            return Ok(());
        }

        // Parse the error to decide how to proceed.
        let err_body = poll_resp.text().await.unwrap_or_default();
        let error_code = serde_json::from_str::<TokenErrorResponse>(&err_body)
            .map(|e| e.error)
            .unwrap_or_else(|_| err_body.clone());

        match error_code.as_str() {
            "authorization_pending" => {
                // Normal – keep polling.
            }
            "slow_down" => {
                interval_secs += 5;
            }
            "access_denied" => {
                bail!("authorization was denied – please run `rw auth login` again");
            }
            "expired_token" => {
                bail!("device code expired – please run `rw auth login` again");
            }
            _ => {
                bail!("unexpected error from token endpoint: {}", err_body);
            }
        }
    }
}

/// Run `rw auth status` – report whether stored credentials exist.
pub fn status(ctx: &AppContext, out: &Output) -> Result<()> {
    match load_auth_cache(&ctx.config_dir, &ctx.auth_profile)? {
        Some(ref cache @ AuthCache::Bearer { .. }) => {
            out.print(&StatusOutput {
                auth_type: Some("bearer".to_string()),
                authenticated: true,
                expired: cache.is_expired(),
                username: None,
                profile: ctx.auth_profile.clone(),
            });
        }
        Some(AuthCache::Basic { ref username, .. }) => {
            out.print(&StatusOutput {
                auth_type: Some("basic".to_string()),
                authenticated: true,
                expired: false,
                username: Some(username.clone()),
                profile: ctx.auth_profile.clone(),
            });
        }
        None => {
            out.print(&StatusOutput {
                auth_type: None,
                authenticated: false,
                expired: false,
                username: None,
                profile: ctx.auth_profile.clone(),
            });
        }
    }
    Ok(())
}

/// Returns the Authorization header value for the given resolved credentials.
pub fn auth_header_value(auth: &ResolvedAuth) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    match auth {
        ResolvedAuth::Bearer(token) => format!("Bearer {}", token),
        ResolvedAuth::Basic { username, password } => {
            let encoded = STANDARD.encode(format!("{}:{}", username, password));
            format!("Basic {}", encoded)
        }
    }
}

/// Run `rw auth header` – print the Authorization header value for API requests.
/// For bearer tokens, prints `Bearer <token>` (refreshing if expired).
/// For basic credentials, prints `Basic <base64(username:password)>`.
pub async fn header(ctx: &AppContext, out: &Output) -> Result<()> {
    match resolve_auth(ctx).await? {
        Some(ref auth) => {
            out.print(&HeaderOutput {
                header: auth_header_value(auth),
            });
        }
        None => {
            anyhow::bail!("not authenticated – run `rw auth login` first");
        }
    }
    Ok(())
}

/// Run `rw auth logout` – remove stored credentials for the profile.
pub fn logout(ctx: &AppContext, out: &Output) -> Result<()> {
    if delete_auth_cache(&ctx.config_dir, &ctx.profile)? {
        out.print(&MessageOutput {
            message: format!("✓ Credentials for profile '{}' removed.", ctx.profile),
        });
    } else {
        out.print(&MessageOutput {
            message: format!("No stored credentials found for profile '{}'.", ctx.profile),
        });
    }
    Ok(())
}

/// Resolved authentication credentials ready to attach to a request.
pub enum ResolvedAuth {
    Bearer(String),
    Basic { username: String, password: String },
}

/// Resolves auth credentials for the given organization+stage, loading the cache once.
/// For bearer tokens, automatically refreshes if expired.
/// Returns `None` if no credentials are stored.
pub async fn resolve_auth(ctx: &AppContext) -> Result<Option<ResolvedAuth>> {
    let Some(cache) = load_auth_cache(&ctx.config_dir, &ctx.auth_profile)? else {
        return Ok(None);
    };

    match cache {
        AuthCache::Basic { username, password } => {
            Ok(Some(ResolvedAuth::Basic { username, password }))
        }
        AuthCache::Bearer {
            ref access_token, ..
        } => {
            if !cache.is_expired() {
                return Ok(Some(ResolvedAuth::Bearer(access_token.clone())));
            }

            // Token is expired – attempt a refresh.
            let refresh_token = match &cache {
                AuthCache::Bearer {
                    refresh_token: Some(rt),
                    ..
                } => rt.clone(),
                _ => bail!("authentication token expired; run `rw auth login` to re-authenticate"),
            };

            let new_cache = try_refresh(&ctx.auth_stage, &refresh_token)
                .await
                .context("token refresh failed; run `rw auth login` to re-authenticate")?;

            save_auth_cache(&ctx.config_dir, &ctx.auth_profile, &new_cache)?;

            match new_cache {
                AuthCache::Bearer { access_token, .. } => {
                    Ok(Some(ResolvedAuth::Bearer(access_token)))
                }
                _ => unreachable!("refresh always returns a bearer token"),
            }
        }
    }
}

/// Exchanges a refresh token for a new access token and refresh token.
async fn try_refresh(stage: &Stage, refresh_token: &str) -> Result<AuthCache> {
    let wos = workos_config(stage);
    let client = reqwest::Client::new();

    let resp = client
        .post(wos.token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", wos.client_id),
        ])
        .send()
        .await
        .context("failed to reach WorkOS token endpoint")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("token endpoint returned {}: {}", status, body);
    }

    let token: TokenResponse = resp
        .json()
        .await
        .context("failed to parse refresh token response")?;

    Ok(AuthCache::Bearer {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_at: expires_at_from_duration(token.expires_in),
    })
}

/// Returns the Authorization header value, or fails with a friendly message if
/// no credentials are stored.
pub async fn require_auth(ctx: &AppContext) -> Result<String> {
    resolve_auth(ctx)
        .await?
        .map(|a| auth_header_value(&a))
        .ok_or_else(|| anyhow::anyhow!("not authenticated – run `rw auth login` first"))
}

/// Attaches stored credentials to `req`. Returns the request unmodified if no
/// credentials are stored (the caller will receive a 401 from the API).
pub async fn attach_auth(
    ctx: &AppContext,
    req: reqwest::RequestBuilder,
) -> Result<reqwest::RequestBuilder> {
    Ok(match resolve_auth(ctx).await? {
        Some(ResolvedAuth::Bearer(token)) => {
            req.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        }
        Some(ResolvedAuth::Basic { username, password }) => {
            req.basic_auth(&username, Some(&password))
        }
        None => req,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header_value_bearer() {
        let auth = ResolvedAuth::Bearer("mytoken123".to_string());
        assert_eq!(auth_header_value(&auth), "Bearer mytoken123");
    }

    #[test]
    fn test_auth_header_value_basic() {
        let auth = ResolvedAuth::Basic {
            username: "alice".to_string(),
            password: "secret".to_string(),
        };
        // base64("alice:secret") = "YWxpY2U6c2VjcmV0"
        assert_eq!(auth_header_value(&auth), "Basic YWxpY2U6c2VjcmV0");
    }

    #[test]
    fn test_auth_header_value_basic_special_chars() {
        let auth = ResolvedAuth::Basic {
            username: "user@example.com".to_string(),
            password: "p@ss:word".to_string(),
        };
        use base64::{engine::general_purpose::STANDARD, Engine};
        let expected = format!("Basic {}", STANDARD.encode("user@example.com:p@ss:word"));
        assert_eq!(auth_header_value(&auth), expected);
    }

    #[test]
    fn test_message_output_plain() {
        let output = MessageOutput {
            message: "✓ Authenticated successfully. Credentials saved for profile 'demo'."
                .to_string(),
        };
        assert_eq!(output.plain(), output.message);
    }

    #[test]
    fn test_status_output_json_authenticated_bearer() {
        let output = StatusOutput {
            auth_type: Some("bearer".to_string()),
            authenticated: true,
            expired: false,
            username: None,
            profile: "demo".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["type"], "bearer");
        assert_eq!(json["authenticated"], true);
        assert_eq!(json["expired"], false);
        // profile is skipped; username is omitted when None
        assert!(json.get("profile").is_none());
        assert!(json.get("username").is_none());
    }

    #[test]
    fn test_status_output_json_authenticated_basic() {
        let output = StatusOutput {
            auth_type: Some("basic".to_string()),
            authenticated: true,
            expired: false,
            username: Some("alice".to_string()),
            profile: "demo".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["type"], "basic");
        assert_eq!(json["username"], "alice");
    }

    #[test]
    fn test_status_output_json_unauthenticated() {
        let output = StatusOutput {
            auth_type: None,
            authenticated: false,
            expired: false,
            username: None,
            profile: "demo".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["type"].is_null());
        assert_eq!(json["authenticated"], false);
    }

    #[test]
    fn test_status_output_plain_bearer() {
        let output = StatusOutput {
            auth_type: Some("bearer".to_string()),
            authenticated: true,
            expired: false,
            username: None,
            profile: "demo".to_string(),
        };
        assert_eq!(
            output.plain(),
            "✓ Authenticated using profile 'demo' (bearer token)."
        );
    }

    #[test]
    fn test_status_output_plain_basic_with_username() {
        let output = StatusOutput {
            auth_type: Some("basic".to_string()),
            authenticated: true,
            expired: false,
            username: Some("alice".to_string()),
            profile: "demo".to_string(),
        };
        assert_eq!(
            output.plain(),
            "✓ Authenticated using profile 'demo' (basic, user: alice)."
        );
    }

    #[test]
    fn test_status_output_plain_unauthenticated() {
        let output = StatusOutput {
            auth_type: None,
            authenticated: false,
            expired: false,
            username: None,
            profile: "demo".to_string(),
        };
        assert!(output.plain().contains("✗ Not authenticated"));
        assert!(output.plain().contains("demo"));
    }

    #[test]
    fn test_header_output_plain() {
        let output = HeaderOutput {
            header: "Bearer mytoken".to_string(),
        };
        assert_eq!(output.plain(), "Bearer mytoken");
    }

    #[tokio::test]
    async fn test_resolve_auth_reads_from_auth_profile() {
        use crate::auth_cache::{save_auth_cache, AuthCache};
        use crate::cli::Stage;
        use std::collections::BTreeMap;

        let dir = tempfile::TempDir::new().unwrap();

        // The active profile has NO credentials.
        // The auth-source profile has Basic credentials.
        save_auth_cache(
            dir.path(),
            "source",
            &AuthCache::Basic {
                username: "alice".to_string(),
                password: "secret".to_string(),
            },
        )
        .unwrap();

        let ctx = AppContext {
            config_dir: dir.path().to_path_buf(),
            profile: "active".to_string(),
            auth_profile: "source".to_string(),
            stage: Stage::Dev,
            auth_stage: Stage::Dev,
            base_url: "http://example".to_string(),
            defaults: BTreeMap::new(),
        };

        let resolved = resolve_auth(&ctx).await.unwrap();
        match resolved {
            Some(ResolvedAuth::Basic { username, password }) => {
                assert_eq!(username, "alice");
                assert_eq!(password, "secret");
            }
            _ => panic!("expected Basic auth from the override profile"),
        }
    }

    #[tokio::test]
    async fn test_resolve_auth_returns_none_when_auth_profile_has_no_credentials() {
        use crate::auth_cache::{save_auth_cache, AuthCache};
        use crate::cli::Stage;
        use std::collections::BTreeMap;

        let dir = tempfile::TempDir::new().unwrap();

        // The active profile DOES have credentials, but the override profile does not.
        save_auth_cache(
            dir.path(),
            "active",
            &AuthCache::Basic {
                username: "alice".to_string(),
                password: "secret".to_string(),
            },
        )
        .unwrap();

        let ctx = AppContext {
            config_dir: dir.path().to_path_buf(),
            profile: "active".to_string(),
            auth_profile: "source".to_string(),
            stage: Stage::Dev,
            auth_stage: Stage::Dev,
            base_url: "http://example".to_string(),
            defaults: BTreeMap::new(),
        };

        let resolved = resolve_auth(&ctx).await.unwrap();
        assert!(
            resolved.is_none(),
            "should not fall back to active profile's credentials"
        );
    }

    #[test]
    fn test_status_reports_auth_profile_in_output() {
        use crate::auth_cache::{save_auth_cache, AuthCache};
        use crate::cli::Stage;
        use std::collections::BTreeMap;

        let dir = tempfile::TempDir::new().unwrap();
        save_auth_cache(
            dir.path(),
            "source",
            &AuthCache::Basic {
                username: "alice".to_string(),
                password: "secret".to_string(),
            },
        )
        .unwrap();

        let ctx = AppContext {
            config_dir: dir.path().to_path_buf(),
            profile: "active".to_string(),
            auth_profile: "source".to_string(),
            stage: Stage::Dev,
            auth_stage: Stage::Dev,
            base_url: "http://example".to_string(),
            defaults: BTreeMap::new(),
        };

        // We can't directly capture Output, but we can re-invoke the same
        // logic and inspect what would have been printed by reading the cache.
        let cache = crate::auth_cache::load_auth_cache(&ctx.config_dir, &ctx.auth_profile)
            .unwrap()
            .unwrap();
        match cache {
            AuthCache::Basic { username, .. } => assert_eq!(username, "alice"),
            _ => panic!("expected Basic"),
        }

        // Also call status directly with a non-JSON output to ensure no panic.
        let out = crate::output::Output { json: false };
        status(&ctx, &out).unwrap();
    }

    #[tokio::test]
    async fn test_attach_auth_uses_override_credentials() {
        use crate::auth_cache::{save_auth_cache, AuthCache};
        use crate::cli::Stage;
        use base64::{engine::general_purpose::STANDARD, Engine};
        use std::collections::BTreeMap;

        let dir = tempfile::TempDir::new().unwrap();

        // Profile A would target this base URL — but its auth file is empty.
        let mut server = mockito::Server::new_async().await;

        // Profile B holds the real credentials we want to use.
        save_auth_cache(
            dir.path(),
            "profile-b",
            &AuthCache::Basic {
                username: "bob".to_string(),
                password: "hunter2".to_string(),
            },
        )
        .unwrap();

        let expected_header = format!("Basic {}", STANDARD.encode("bob:hunter2"));

        let mock = server
            .mock("GET", "/ping")
            .match_header("authorization", expected_header.as_str())
            .with_status(200)
            .with_body("ok")
            .create_async()
            .await;

        // Active profile = A (its URL drives the request).
        // Auth source = B (its credentials are attached).
        let ctx = AppContext {
            config_dir: dir.path().to_path_buf(),
            profile: "profile-a".to_string(),
            auth_profile: "profile-b".to_string(),
            stage: Stage::Dev,
            auth_stage: Stage::Dev,
            base_url: server.url(),
            defaults: BTreeMap::new(),
        };

        let client = reqwest::Client::new();
        let req = client.get(format!("{}/ping", ctx.base_url));
        let req = attach_auth(&ctx, req).await.unwrap();
        let resp = req.send().await.unwrap();
        assert!(resp.status().is_success());

        mock.assert_async().await;
    }
}
