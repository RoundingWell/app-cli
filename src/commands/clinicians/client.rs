//! Legacy reqwest helpers for clinician lookups.
//!
//! These pre-date the introduction of `crate::http::ApiClient` and will be
//! folded into the ApiClient migration in a follow-up. For now they remain
//! as free functions sharing a `client`, `base_url`, `auth_header` triple.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use uuid::Uuid;

use crate::jsonapi::List;

use super::data::{ClinicianListResponse, ClinicianResource, ClinicianSingleResponse};

pub(super) fn apply_auth(
    req: reqwest::RequestBuilder,
    auth_header: &str,
) -> reqwest::RequestBuilder {
    req.header(reqwest::header::AUTHORIZATION, auth_header)
}

pub(super) async fn resolve_me(
    client: &Client,
    base_url: &str,
    auth_header: &str,
) -> Result<String> {
    let url = format!("{}/clinicians/me", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /clinicians/me failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;
    Ok(response.data.id)
}

pub(super) async fn fetch_clinician_me(
    client: &Client,
    base_url: &str,
    auth_header: &str,
) -> Result<ClinicianResource> {
    let url = format!("{}/clinicians/me", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /clinicians/me failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;
    Ok(response.data)
}

pub(super) async fn fetch_clinician_by_email_filter(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    email: &str,
) -> Result<ClinicianResource> {
    let url = format!("{}/clinicians", base_url.trim_end_matches('/'));
    let req = apply_auth(
        client.get(&url).query(&[("filter[email]", email)]),
        auth_header,
    );
    let resp = req
        .send()
        .await
        .context("GET /clinicians?filter[email] failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let list: ClinicianListResponse =
        serde_json::from_str(&body).context("failed to parse clinicians response")?;
    list.data
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no clinician found with email {}", email))
}

pub(super) async fn resolve_uuid_by_email(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    email: &str,
) -> Result<String> {
    fetch_clinician_by_email(client, base_url, auth_header, email)
        .await
        .map(|c| c.id)
}

pub(super) async fn fetch_clinician_by_uuid(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    uuid: &str,
) -> Result<ClinicianResource> {
    let url = format!("{}/clinicians/{}", base_url.trim_end_matches('/'), uuid);
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req
        .send()
        .await
        .with_context(|| format!("GET /clinicians/{} failed", uuid))?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;
    Ok(response.data)
}

pub(super) async fn fetch_clinician_by_email(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    email: &str,
) -> Result<ClinicianResource> {
    let email_lower = email.to_lowercase();
    let url = format!("{}/clinicians", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /clinicians failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let list: ClinicianListResponse =
        serde_json::from_str(&body).context("failed to parse clinicians response")?;
    list.data
        .into_iter()
        .find(|c| c.attributes.email.to_lowercase() == email_lower)
        .ok_or_else(|| anyhow::anyhow!("no clinician found with email {}", email))
}

pub(super) async fn resolve_team(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    target: &str,
) -> Result<(String, String)> {
    use crate::commands::teams::TeamAttributes;

    let url = format!("{}/teams", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /teams failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    let list: List<TeamAttributes> =
        serde_json::from_str(&body).context("failed to parse teams response")?;
    let target_lower = target.to_lowercase();

    if Uuid::parse_str(target).is_ok() {
        list.data
            .into_iter()
            .find(|t| t.id.to_lowercase() == target_lower)
            .map(|t| (t.id, t.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no team found with id '{}'", target))
    } else {
        list.data
            .into_iter()
            .find(|t| t.attributes.abbr.to_lowercase() == target_lower)
            .map(|t| (t.id, t.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no team found with uuid or abbr '{}'", target))
    }
}
