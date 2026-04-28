//! Lookup helpers shared across clinician commands. All take an `&ApiClient`
//! that has already resolved auth and carries the base URL.

use anyhow::Result;
use uuid::Uuid;

use crate::http::ApiClient;
use crate::jsonapi::{List, Single};

use super::data::{Clinician, ClinicianAttributes};

pub(super) async fn resolve_me(api: &ApiClient<'_>) -> Result<String> {
    let resp: Single<ClinicianAttributes> = api.get("clinicians/me").await?;
    Ok(resp.data.id)
}

pub(super) async fn fetch_clinician_me(api: &ApiClient<'_>) -> Result<Clinician> {
    let resp: Single<ClinicianAttributes> = api.get("clinicians/me").await?;
    Ok(resp.data)
}

pub(super) async fn fetch_clinician_by_email_filter(
    api: &ApiClient<'_>,
    email: &str,
) -> Result<Clinician> {
    let resp: List<ClinicianAttributes> = api
        .get_query("clinicians", &[("filter[email]", email)])
        .await?;
    resp.data
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no clinician found with email {}", email))
}

pub(super) async fn fetch_clinician_by_uuid(api: &ApiClient<'_>, uuid: &str) -> Result<Clinician> {
    let resp: Single<ClinicianAttributes> = api.get(&format!("clinicians/{}", uuid)).await?;
    Ok(resp.data)
}

pub(super) async fn fetch_clinician_by_email(
    api: &ApiClient<'_>,
    email: &str,
) -> Result<Clinician> {
    let email_lower = email.to_lowercase();
    let resp: List<ClinicianAttributes> = api.get("clinicians").await?;
    resp.data
        .into_iter()
        .find(|c| c.attributes.email.to_lowercase() == email_lower)
        .ok_or_else(|| anyhow::anyhow!("no clinician found with email {}", email))
}

pub(super) async fn resolve_uuid_by_email(api: &ApiClient<'_>, email: &str) -> Result<String> {
    fetch_clinician_by_email(api, email).await.map(|c| c.id)
}

pub(super) async fn resolve_team(api: &ApiClient<'_>, target: &str) -> Result<(String, String)> {
    use crate::commands::teams::TeamAttributes;

    let resp: List<TeamAttributes> = api.get("teams").await?;
    let target_lower = target.to_lowercase();

    if Uuid::parse_str(target).is_ok() {
        resp.data
            .into_iter()
            .find(|t| t.id.to_lowercase() == target_lower)
            .map(|t| (t.id, t.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no team found with id '{}'", target))
    } else {
        resp.data
            .into_iter()
            .find(|t| t.attributes.abbr.to_lowercase() == target_lower)
            .map(|t| (t.id, t.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no team found with uuid or abbr '{}'", target))
    }
}
