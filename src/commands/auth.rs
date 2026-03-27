use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;

use crate::config::{load_config, save_config, AuthEntry};

/// WorkOS OAuth client ID for the RoundingWell application.
const CLIENT_ID: &str = "client_01KMREY0MMNCB4B9AK4X9C0TBG";

/// WorkOS authorization endpoint.
const AUTH_URL: &str = "https://api.workos.com/user_management/authorize";

/// WorkOS token exchange endpoint.
const TOKEN_URL: &str = "https://api.workos.com/user_management/authenticate";

/// Response body from the WorkOS token endpoint.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

/// Parsed query parameters from an OAuth callback URL.
struct CallbackParams {
    code: String,
    state: String,
}

/// Generate a cryptographically random PKCE code verifier (32 bytes → 43-char
/// base64url string).
fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Derive the PKCE code challenge from the verifier using S256.
fn generate_code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

/// Generate a random state value to protect against CSRF.
fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Bind a TCP listener on `127.0.0.1` at an OS-assigned port and return it
/// together with the port number.
fn bind_local_listener() -> Result<(TcpListener, u16)> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .context("could not bind local listener for OAuth callback")?;
    let port = listener.local_addr()?.port();
    Ok((listener, port))
}

/// Wait for a single HTTP request on `listener` and extract the `code` and
/// `state` query parameters from the request path.
fn wait_for_callback(listener: TcpListener) -> Result<CallbackParams> {
    let (mut stream, _) = listener
        .accept()
        .context("error accepting OAuth callback connection")?;

    let mut request = String::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = stream.read(&mut buf).context("error reading HTTP request")?;
        request.push_str(&String::from_utf8_lossy(&buf[..n]));
        if request.contains("\r\n\r\n") || n == 0 {
            break;
        }
    }

    // Parse "GET /callback?code=xxx&state=yyy HTTP/1.1"
    let first_line = request.lines().next().unwrap_or_default();
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default();

    let query = path.split('?').nth(1).unwrap_or_default();

    let mut code = None;
    let mut state = None;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        match key {
            "code" => code = Some(percent_decode(value)),
            "state" => state = Some(percent_decode(value)),
            _ => {}
        }
    }

    // Respond with a friendly page regardless of outcome so the browser
    // doesn't show an empty response.
    let body = "<html><body><h2>Authentication complete.</h2>\
                <p>You can close this tab and return to your terminal.</p>\
                </body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());

    match (code, state) {
        (Some(c), Some(s)) => Ok(CallbackParams { code: c, state: s }),
        _ => bail!("OAuth callback did not contain expected 'code' and 'state' parameters"),
    }
}

/// Minimal percent-decode for OAuth callback query values.
/// Collects raw bytes first then converts to UTF-8 to handle multi-byte
/// sequences correctly.
fn percent_decode(s: &str) -> String {
    let mut bytes: Vec<u8> = Vec::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                bytes.push(byte);
            }
        } else if c == '+' {
            bytes.push(b' ');
        } else {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Exchange an authorization code for an access token.
async fn exchange_code(
    client: &reqwest::Client,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    let params = [
        ("client_id", CLIENT_ID),
        ("code", code),
        ("code_verifier", code_verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .context("failed to reach WorkOS token endpoint")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("WorkOS token endpoint returned {}: {}", status, body);
    }

    let token: TokenResponse = resp
        .json()
        .await
        .context("failed to parse WorkOS token response")?;
    Ok(token)
}

/// Run `rw auth login` – open the browser for WorkOS AuthKit authentication,
/// wait for the callback, exchange the code, and persist the bearer token.
pub async fn login(organization: &str) -> Result<()> {
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_state();

    let (listener, port) = bind_local_listener()?;
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    // Build the WorkOS authorization URL.
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code\
         &code_challenge={}&code_challenge_method=S256\
         &provider=authkit\
         &state={}",
        AUTH_URL,
        CLIENT_ID,
        urlencoded(&redirect_uri),
        code_challenge,
        state,
    );

    println!("Opening your browser to authenticate with RoundingWell...");
    println!("If the browser does not open automatically, visit:\n  {}", auth_url);

    if let Err(e) = open::that(&auth_url) {
        eprintln!("Warning: could not open browser automatically: {}", e);
    }

    println!("Waiting for authentication callback...");
    let params = wait_for_callback(listener)?;

    if params.state != state {
        bail!("OAuth state mismatch – possible CSRF attack, aborting");
    }

    let client = reqwest::Client::new();
    let token = exchange_code(&client, &params.code, &code_verifier, &redirect_uri).await?;

    // Persist the bearer token under the organization name.
    let mut config = load_config()?;
    config.authentication.insert(
        organization.to_string(),
        AuthEntry::Bearer {
            bearer: token.access_token,
        },
    );
    save_config(&config)?;

    println!(
        "✓ Authenticated successfully. Credentials saved for organization \"{}\".",
        organization
    );
    Ok(())
}

/// Run `rw auth status` – report whether stored credentials exist.
pub fn status(organization: &str) -> Result<()> {
    let config = load_config()?;
    match config.authentication.get(organization) {
        Some(AuthEntry::Bearer { .. }) => {
            println!("✓ Authenticated to \"{}\" (bearer token).", organization);
        }
        Some(AuthEntry::Basic { basic }) => {
            println!(
                "✓ Authenticated to \"{}\" (basic, user: {}).",
                organization, basic.username
            );
        }
        None => {
            println!("✗ Not authenticated for organization \"{}\".", organization);
            println!("  Run `rw auth login` to authenticate.");
        }
    }
    Ok(())
}

/// Run `rw auth logout` – remove stored credentials for the organization.
pub fn logout(organization: &str) -> Result<()> {
    let mut config = load_config()?;
    if config.authentication.remove(organization).is_some() {
        save_config(&config)?;
        println!(
            "✓ Credentials for organization \"{}\" removed.",
            organization
        );
    } else {
        println!(
            "No stored credentials found for organization \"{}\".",
            organization
        );
    }
    Ok(())
}

/// Percent-encode a string for use in a URL query parameter value.
fn urlencoded(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}
