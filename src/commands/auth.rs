use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::auth_cache::{
    delete_auth_cache, expires_at_from_duration, load_auth_cache, save_auth_cache, AuthCache,
};
use crate::cli::Stage;

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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

/// Error response from the token endpoint while polling.
#[derive(Debug, Deserialize)]
struct TokenErrorResponse {
    error: String,
}

/// Run `rw auth login` – use the OAuth Device Authorization Flow to authenticate
/// via WorkOS AuthKit, poll for a token, and persist credentials.
pub async fn login(profile: &str, organization: &str, stage: &Stage) -> Result<()> {
    let wos = workos_config(stage);
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

    // Step 2: Prompt user to authenticate in browser.
    println!(
        "Open the following URL in your browser and enter code: {}",
        device_auth.user_code
    );
    println!("  {}", device_auth.verification_uri);
    println!();
    println!("Or visit this URL to authenticate automatically:");
    println!("  {}", device_auth.verification_uri_complete);

    if let Err(e) = open::that(&device_auth.verification_uri_complete) {
        eprintln!("Warning: could not open browser automatically: {}", e);
    }

    // Step 3: Poll for the token.
    let mut interval_secs = device_auth.interval.max(5);
    let deadline = std::time::Instant::now() + Duration::from_secs(device_auth.expires_in);

    println!("Waiting for authentication...");

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
            save_auth_cache(organization, stage, &cache)?;

            println!(
                "✓ Authenticated successfully. Credentials saved for profile \"{}\".",
                profile
            );
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
pub fn status(profile: &str, organization: &str, stage: &Stage) -> Result<()> {
    match load_auth_cache(organization, stage)? {
        Some(ref cache @ AuthCache::Bearer { .. }) => {
            if cache.is_expired() {
                println!(
                    "✓ Authenticated using profile \"{}\" (bearer token, expired – will refresh on next use).",
                    profile
                );
            } else {
                println!(
                    "✓ Authenticated using profile \"{}\" (bearer token).",
                    profile
                );
            }
        }
        Some(AuthCache::Basic { username, .. }) => {
            println!(
                "✓ Authenticated using profile \"{}\" (basic, user: {}).",
                profile, username
            );
        }
        None => {
            println!("✗ Not authenticated for profile \"{}\".", profile);
            println!("  Run `rw auth login` to authenticate.");
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
pub async fn header(organization: &str, stage: &Stage) -> Result<()> {
    match resolve_auth(organization, stage).await? {
        Some(ref auth) => {
            println!("{}", auth_header_value(auth));
        }
        None => {
            anyhow::bail!("not authenticated – run `rw auth login` first");
        }
    }
    Ok(())
}

/// Run `rw auth logout` – remove stored credentials for the profile.
pub fn logout(profile: &str, organization: &str, stage: &Stage) -> Result<()> {
    if delete_auth_cache(organization, stage)? {
        println!("✓ Credentials for profile \"{}\" removed.", profile);
    } else {
        println!("No stored credentials found for profile \"{}\".", profile);
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
pub async fn resolve_auth(organization: &str, stage: &Stage) -> Result<Option<ResolvedAuth>> {
    let Some(cache) = load_auth_cache(organization, stage)? else {
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

            let new_cache = try_refresh(stage, &refresh_token)
                .await
                .context("token refresh failed; run `rw auth login` to re-authenticate")?;

            save_auth_cache(organization, stage, &new_cache)?;

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
}
