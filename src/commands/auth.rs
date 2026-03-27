use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::cli::Stage;
use crate::config::{load_config, save_config, AuthEntry};

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
}

/// Error response from the token endpoint while polling.
#[derive(Debug, Deserialize)]
struct TokenErrorResponse {
    error: String,
}

/// Run `rw auth login` – use the OAuth Device Authorization Flow to authenticate
/// via WorkOS AuthKit, poll for a token, and persist the bearer token.
pub async fn login(profile: &str, stage: &Stage) -> Result<()> {
    let wos = workos_config(stage);
    let client = reqwest::Client::new();

    // Step 1: Request device authorization.
    let resp = client
        .post(wos.device_auth_url)
        .form(&[("client_id", wos.client_id)])
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

            let mut config = load_config()?;
            config.authentication.insert(
                profile.to_string(),
                AuthEntry::Bearer {
                    bearer: token.access_token,
                },
            );
            save_config(&config)?;

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
/// When `show` is true, print the raw credential value.
pub fn status(profile: &str, show: bool) -> Result<()> {
    let config = load_config()?;
    match config.authentication.get(profile) {
        Some(AuthEntry::Bearer { bearer }) => {
            println!(
                "✓ Authenticated using profile \"{}\" (bearer token).",
                profile
            );
            if show {
                println!("  token: {}", bearer);
            }
        }
        Some(AuthEntry::Basic { basic }) => {
            println!(
                "✓ Authenticated using profile \"{}\" (basic, user: {}).",
                profile, basic.username
            );
            if show {
                println!("  username: {}", basic.username);
                println!("  password: {}", basic.password);
            }
        }
        None => {
            println!("✗ Not authenticated for profile \"{}\".", profile);
            println!("  Run `rw auth login` to authenticate.");
        }
    }
    Ok(())
}

/// Run `rw auth logout` – remove stored credentials for the profile.
pub fn logout(profile: &str) -> Result<()> {
    let mut config = load_config()?;
    if config.authentication.remove(profile).is_some() {
        save_config(&config)?;
        println!("✓ Credentials for profile \"{}\" removed.", profile);
    } else {
        println!("No stored credentials found for profile \"{}\".", profile);
    }
    Ok(())
}
