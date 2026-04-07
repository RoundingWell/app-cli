use anyhow::{bail, Context, Result};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use uuid::Uuid;
use validator::Validate;

use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

// --- Validation ---

static NPI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{10}$").unwrap());

#[derive(Validate)]
struct ClinicianUpdateInput {
    #[validate(length(min = 1))]
    name: Option<String>,
    #[validate(email)]
    email: Option<String>,
    #[validate(length(equal = 10), regex(path = *NPI_RE))]
    npi: Option<String>,
}

const ALLOWED_FIELDS: &[&str] = &["name", "email", "npi", "credentials"];

fn validate_field(field: &str, value: Option<&str>) -> Result<()> {
    match field {
        "name" => {
            let v = value.unwrap_or("").trim().to_string();
            let input = ClinicianUpdateInput {
                name: Some(v),
                email: None,
                npi: None,
            };
            input
                .validate()
                .map_err(|e| anyhow::anyhow!("invalid name: {}", e))?;
        }
        "email" => {
            let v = value.unwrap_or("").to_string();
            let input = ClinicianUpdateInput {
                name: None,
                email: Some(v),
                npi: None,
            };
            input
                .validate()
                .map_err(|e| anyhow::anyhow!("invalid email: {}", e))?;
        }
        "npi" => {
            if let Some(v) = value {
                if !v.is_empty() {
                    let input = ClinicianUpdateInput {
                        name: None,
                        email: None,
                        npi: Some(v.to_string()),
                    };
                    input
                        .validate()
                        .map_err(|e| anyhow::anyhow!("invalid NPI: {}", e))?;
                }
            }
        }
        "credentials" => {}
        _ => {
            bail!(
                "unsupported field '{}'; allowed fields: {}",
                field,
                ALLOWED_FIELDS.join(", ")
            );
        }
    }
    Ok(())
}

fn build_attribute_value(field: &str, value: Option<&str>) -> serde_json::Value {
    match field {
        "credentials" => match value {
            None | Some("") => serde_json::json!([]),
            Some(v) => {
                let parts: Vec<&str> = v.split(',').collect();
                serde_json::json!(parts)
            }
        },
        "npi" => match value {
            None | Some("") => serde_json::Value::Null,
            Some(v) => serde_json::json!(v),
        },
        _ => serde_json::json!(value.unwrap_or("")),
    }
}

// --- JSON:API deserialization types ---

#[derive(Debug, Deserialize)]
struct ClinicianAttributes {
    name: String,
    email: String,
    enabled: bool,
    #[serde(default)]
    npi: Option<String>,
    #[serde(default)]
    credentials: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClinicianResource {
    id: String,
    attributes: ClinicianAttributes,
}

#[derive(Debug, Deserialize)]
struct ClinicianListResponse {
    data: Vec<ClinicianResource>,
}

#[derive(Debug, Deserialize)]
struct ClinicianSingleResponse {
    data: ClinicianResource,
}

use super::teams::TeamListResponse;

#[derive(Debug, Deserialize)]
struct WorkspaceSettings {
    #[serde(default)]
    default_for_clinicians: bool,
}

#[derive(Debug, Deserialize)]
struct WorkspaceAttributes {
    settings: WorkspaceSettings,
}

#[derive(Debug, Deserialize)]
struct WorkspaceResource {
    id: String,
    attributes: WorkspaceAttributes,
}

#[derive(Debug, Deserialize)]
struct WorkspaceListResponse {
    data: Vec<WorkspaceResource>,
}

// --- Output types ---

#[derive(Debug, Serialize)]
pub struct PrepareOutput {
    #[serde(rename = "id")]
    pub clinician_id: String,
    #[serde(rename = "name")]
    pub clinician_name: String,
    pub is_staff: bool,
    pub role_id: String,
    pub role_name: String,
    pub team_id: String,
    pub team_name: String,
    pub hidden: bool,
    pub workspace_ids: Vec<String>,
}

impl CommandOutput for PrepareOutput {
    fn plain(&self) -> String {
        let kind = if self.is_staff { "staff" } else { "employee" };
        let ws = self.workspace_ids.join(", ");
        format!(
            "{} ({}) prepared as {}: role={}, team={}, hidden={}, workspaces=[{}]",
            self.clinician_name,
            self.clinician_id,
            kind,
            self.role_name,
            self.team_name,
            self.hidden,
            ws
        )
    }
}

#[derive(Debug, Serialize)]
pub struct GrantOutput {
    pub clinician_id: String,
    pub clinician_name: String,
    pub role_id: String,
    pub role_name: String,
}

impl CommandOutput for GrantOutput {
    fn plain(&self) -> String {
        format!(
            "{} ({}) granted '{}' role",
            self.clinician_name, self.clinician_id, self.role_name
        )
    }
}

#[derive(Debug, Serialize)]
pub struct AssignTeamOutput {
    pub clinician_id: String,
    pub clinician_name: String,
    pub team_id: String,
    pub team_name: String,
}

impl CommandOutput for AssignTeamOutput {
    fn plain(&self) -> String {
        format!(
            "{} ({}) assigned to '{}' team",
            self.clinician_name, self.clinician_id, self.team_name
        )
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianOutput {
    pub id: String,
    pub name: String,
    pub email: String,
    pub enabled: bool,
}

impl CommandOutput for ClinicianOutput {
    fn plain(&self) -> String {
        let status = if self.enabled { "enabled" } else { "disabled" };
        format!("{} ({}) is now {}", self.name, self.id, status)
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianUpdateOutput {
    pub id: String,
    pub name: String,
    pub email: String,
    pub enabled: bool,
    pub npi: Option<String>,
    pub credentials: Vec<String>,
    #[serde(skip)]
    pub updated_field: String,
}

impl CommandOutput for ClinicianUpdateOutput {
    fn plain(&self) -> String {
        format!("{} ({}) updated {}", self.name, self.id, self.updated_field)
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianRegisterOutput {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl CommandOutput for ClinicianRegisterOutput {
    fn plain(&self) -> String {
        format!("{} ({}) registered", self.name, self.id)
    }
}

// --- Public command functions ---

pub async fn grant(ctx: &AppContext, target: &str, role_target: &str, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();
    let clinician_uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&client, &ctx.base_url, &auth_header, target).await?
    };
    let (role_id, role_name) =
        resolve_role(&client, &ctx.base_url, &auth_header, role_target).await?;
    let (clinician_id, clinician_name) = patch_clinician_role(
        &client,
        &ctx.base_url,
        &auth_header,
        &clinician_uuid,
        &role_id,
    )
    .await?;
    out.print(&GrantOutput {
        clinician_id,
        clinician_name,
        role_id,
        role_name,
    });
    Ok(())
}

pub async fn assign(ctx: &AppContext, target: &str, team_target: &str, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();
    let clinician_uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&client, &ctx.base_url, &auth_header, target).await?
    };
    let (team_id, team_name) =
        resolve_team(&client, &ctx.base_url, &auth_header, team_target).await?;
    let (clinician_id, clinician_name) = patch_clinician_team(
        &client,
        &ctx.base_url,
        &auth_header,
        &clinician_uuid,
        &team_id,
    )
    .await?;
    out.print(&AssignTeamOutput {
        clinician_id,
        clinician_name,
        team_id,
        team_name,
    });
    Ok(())
}

pub async fn prepare(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    // Step 1: Resolve clinician UUID and email
    let clinician = if Uuid::parse_str(target).is_ok() {
        fetch_clinician_by_uuid(&client, &ctx.base_url, &auth_header, target).await?
    } else {
        fetch_clinician_by_email(&client, &ctx.base_url, &auth_header, target).await?
    };
    let clinician_uuid = clinician.id.clone();
    let clinician_name = clinician.attributes.name.clone();
    let email = clinician.attributes.email.clone();

    // Step 2: Determine staff status
    let is_staff = email.to_lowercase().ends_with("@roundingwell.com");

    // Step 3: Derive configuration
    let (role_name_target, team_name_target, hidden) = if is_staff {
        ("rw", "OT", true)
    } else {
        let role = ctx
            .defaults
            .get("role")
            .map(String::as_str)
            .unwrap_or("employee");
        let team = ctx
            .defaults
            .get("team")
            .map(String::as_str)
            .unwrap_or("NUR");
        (role, team, false)
    };

    // Step 4: Resolve role UUID
    let (role_id, role_name) =
        resolve_role(&client, &ctx.base_url, &auth_header, role_name_target).await?;

    // Step 5: Resolve team UUID
    let (team_id, team_name) =
        resolve_team(&client, &ctx.base_url, &auth_header, team_name_target).await?;

    // Step 6: Fetch default workspace UUIDs
    let workspace_ids =
        fetch_default_clinician_workspace_uuids(&client, &ctx.base_url, &auth_header).await?;

    // Step 7: PATCH clinician
    patch_clinician_prepare(
        &client,
        &ctx.base_url,
        &auth_header,
        &clinician_uuid,
        &role_id,
        &team_id,
        hidden,
    )
    .await?;

    // Step 8: Add to default workspaces; failures are warnings, not fatal errors
    let mut added_workspace_ids = Vec::new();
    for ws_uuid in &workspace_ids {
        match add_clinician_to_workspace(
            &client,
            &ctx.base_url,
            &auth_header,
            ws_uuid,
            &clinician_uuid,
        )
        .await
        {
            Ok(()) => added_workspace_ids.push(ws_uuid.clone()),
            Err(e) => out.warn(&format!(
                "warning: failed to add clinician to workspace {}: {:#}",
                ws_uuid, e
            )),
        }
    }

    out.print(&PrepareOutput {
        clinician_id: clinician_uuid,
        clinician_name,
        is_staff,
        role_id,
        role_name,
        team_id,
        team_name,
        hidden,
        workspace_ids: added_workspace_ids,
    });
    Ok(())
}

pub async fn enable(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    set_enabled(ctx, target, true, out).await
}

pub async fn disable(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    set_enabled(ctx, target, false, out).await
}

pub async fn update(
    ctx: &AppContext,
    target: &str,
    field: &str,
    value: Option<&str>,
    out: &Output,
) -> Result<()> {
    validate_field(field, value)?;

    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    let uuid = if target == "me" {
        resolve_me(&client, &ctx.base_url, &auth_header).await?
    } else if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&client, &ctx.base_url, &auth_header, target).await?
    };

    let result =
        patch_clinician_attribute(&client, &ctx.base_url, &auth_header, &uuid, field, value)
            .await?;
    out.print(&result);
    Ok(())
}

pub async fn register(
    ctx: &AppContext,
    email: &str,
    name: &str,
    role_target: Option<&str>,
    team_target: Option<&str>,
    out: &Output,
) -> Result<()> {
    // Validate inputs before any API call
    validate_field("name", Some(name))?;
    validate_field("email", Some(email))?;

    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    // Resolve role and team before POST
    let role = if let Some(rt) = role_target {
        Some(resolve_role(&client, &ctx.base_url, &auth_header, rt).await?)
    } else {
        None
    };
    let team = if let Some(tt) = team_target {
        Some(resolve_team(&client, &ctx.base_url, &auth_header, tt).await?)
    } else {
        None
    };

    // Build JSON:API POST body
    let mut data = serde_json::json!({
        "type": "clinicians",
        "attributes": {
            "email": email,
            "name": name
        }
    });

    if role.is_some() || team.is_some() {
        let mut relationships = serde_json::Map::new();
        if let Some((ref role_id, _)) = role {
            relationships.insert(
                "role".to_string(),
                serde_json::json!({"data": {"type": "roles", "id": role_id}}),
            );
        }
        if let Some((ref team_id, _)) = team {
            relationships.insert(
                "team".to_string(),
                serde_json::json!({"data": {"type": "teams", "id": team_id}}),
            );
        }
        data["relationships"] = serde_json::Value::Object(relationships);
    }

    let body = serde_json::json!({ "data": data });

    let url = format!("{}/clinicians", ctx.base_url.trim_end_matches('/'));
    let req = apply_auth(client.post(&url), &auth_header).json(&body);
    let resp = req.send().await.context("POST /clinicians failed")?;
    let status = resp.status();
    let body_text = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body_text);
    }

    let response: ClinicianSingleResponse =
        serde_json::from_str(&body_text).context("failed to parse clinician response")?;

    out.print(&ClinicianRegisterOutput {
        id: response.data.id,
        name: response.data.attributes.name,
        email: response.data.attributes.email,
    });
    Ok(())
}

// --- Private helpers ---

async fn set_enabled(ctx: &AppContext, target: &str, enabled: bool, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();
    let uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&client, &ctx.base_url, &auth_header, target).await?
    };
    let result = patch_clinician(&client, &ctx.base_url, &auth_header, &uuid, enabled).await?;
    out.print(&result);
    Ok(())
}

fn apply_auth(req: reqwest::RequestBuilder, auth_header: &str) -> reqwest::RequestBuilder {
    req.header(reqwest::header::AUTHORIZATION, auth_header)
}

async fn resolve_me(client: &Client, base_url: &str, auth_header: &str) -> Result<String> {
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

async fn patch_clinician_attribute(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    uuid: &str,
    field: &str,
    value: Option<&str>,
) -> Result<ClinicianUpdateOutput> {
    let url = format!("{}/clinicians/{}", base_url.trim_end_matches('/'), uuid);

    let attr_value = build_attribute_value(field, value);
    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": uuid,
            "attributes": {
                (field): attr_value
            }
        }
    });

    let req = apply_auth(client.patch(&url), auth_header).json(&body);
    let resp = req.send().await.context("PATCH /clinicians failed")?;
    let status = resp.status();
    let body_text = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body_text);
    }

    let response: ClinicianSingleResponse =
        serde_json::from_str(&body_text).context("failed to parse clinician response")?;

    Ok(ClinicianUpdateOutput {
        id: response.data.id,
        name: response.data.attributes.name,
        email: response.data.attributes.email,
        enabled: response.data.attributes.enabled,
        npi: response.data.attributes.npi,
        credentials: response.data.attributes.credentials,
        updated_field: field.to_string(),
    })
}

async fn resolve_uuid_by_email(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    email: &str,
) -> Result<String> {
    fetch_clinician_by_email(client, base_url, auth_header, email)
        .await
        .map(|c| c.id)
}

async fn patch_clinician(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    uuid: &str,
    enabled: bool,
) -> Result<ClinicianOutput> {
    let url = format!("{}/clinicians/{}", base_url.trim_end_matches('/'), uuid);

    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": uuid,
            "attributes": {
                "enabled": enabled
            }
        }
    });

    let req = apply_auth(client.patch(&url), auth_header).json(&body);

    let resp = req.send().await.context("PATCH /clinicians failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;

    Ok(ClinicianOutput {
        id: response.data.id,
        name: response.data.attributes.name,
        email: response.data.attributes.email,
        enabled: response.data.attributes.enabled,
    })
}

async fn resolve_role(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    role_target: &str,
) -> Result<(String, String)> {
    let url = format!("{}/roles", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /roles failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    let list: super::roles::RoleListResponse =
        serde_json::from_str(&body).context("failed to parse roles response")?;
    let target_lower = role_target.to_lowercase();

    if Uuid::parse_str(role_target).is_ok() {
        list.data
            .into_iter()
            .find(|r| r.id.to_lowercase() == target_lower)
            .map(|r| (r.id, r.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no role found with id {}", role_target))
    } else {
        list.data
            .into_iter()
            .find(|r| r.attributes.name.to_lowercase() == target_lower)
            .map(|r| (r.id, r.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no role found with name '{}'", role_target))
    }
}

async fn patch_clinician_role(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    clinician_uuid: &str,
    role_uuid: &str,
) -> Result<(String, String)> {
    let url = format!(
        "{}/clinicians/{}",
        base_url.trim_end_matches('/'),
        clinician_uuid
    );

    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": clinician_uuid,
            "relationships": {
                "role": {
                    "data": {
                        "type": "roles",
                        "id": role_uuid
                    }
                }
            }
        }
    });

    let req = apply_auth(client.patch(&url), auth_header).json(&body);

    let resp = req.send().await.context("PATCH /clinicians failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;

    Ok((response.data.id, response.data.attributes.name))
}

async fn patch_clinician_team(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    clinician_uuid: &str,
    team_uuid: &str,
) -> Result<(String, String)> {
    let url = format!(
        "{}/clinicians/{}",
        base_url.trim_end_matches('/'),
        clinician_uuid
    );

    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": clinician_uuid,
            "relationships": {
                "team": {
                    "data": {
                        "type": "teams",
                        "id": team_uuid
                    }
                }
            }
        }
    });

    let req = apply_auth(client.patch(&url), auth_header).json(&body);

    let resp = req.send().await.context("PATCH /clinicians failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;

    Ok((response.data.id, response.data.attributes.name))
}

async fn fetch_clinician_by_uuid(
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

async fn fetch_clinician_by_email(
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

async fn resolve_team(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    target: &str,
) -> Result<(String, String)> {
    let url = format!("{}/teams", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /teams failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    let list: TeamListResponse =
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

async fn fetch_default_clinician_workspace_uuids(
    client: &Client,
    base_url: &str,
    auth_header: &str,
) -> Result<Vec<String>> {
    let url = format!("{}/workspaces", base_url.trim_end_matches('/'));
    let req = apply_auth(client.get(&url), auth_header);
    let resp = req.send().await.context("GET /workspaces failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let list: WorkspaceListResponse =
        serde_json::from_str(&body).context("failed to parse workspaces response")?;
    Ok(list
        .data
        .into_iter()
        .filter(|w| w.attributes.settings.default_for_clinicians)
        .map(|w| w.id)
        .collect())
}

async fn patch_clinician_prepare(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    uuid: &str,
    role_uuid: &str,
    team_uuid: &str,
    hidden: bool,
) -> Result<ClinicianResource> {
    let url = format!("{}/clinicians/{}", base_url.trim_end_matches('/'), uuid);
    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": uuid,
            "attributes": {
                "hidden": hidden
            },
            "relationships": {
                "role": {
                    "data": { "type": "roles", "id": role_uuid }
                },
                "team": {
                    "data": { "type": "teams", "id": team_uuid }
                }
            }
        }
    });
    let req = apply_auth(client.patch(&url), auth_header).json(&body);
    let resp = req.send().await.context("PATCH /clinicians failed")?;
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }
    let response: ClinicianSingleResponse =
        serde_json::from_str(&body).context("failed to parse clinician response")?;
    Ok(response.data)
}

async fn add_clinician_to_workspace(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    workspace_uuid: &str,
    clinician_uuid: &str,
) -> Result<()> {
    let url = format!(
        "{}/workspaces/{}/relationships/clinicians",
        base_url.trim_end_matches('/'),
        workspace_uuid
    );
    let body = serde_json::json!({
        "data": [
            { "type": "clinicians", "id": clinician_uuid }
        ]
    });
    let req = apply_auth(client.post(&url), auth_header).json(&body);
    let resp = req
        .send()
        .await
        .context("POST /workspaces/.../relationships/clinicians failed")?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.context("failed to read response body")?;
        bail!("API returned {}: {}", status, body);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    /// Writes a fake Bearer token to a temp config dir so that `require_auth` succeeds.
    /// The temp directory (and all its contents) is cleaned up on drop.
    struct TestAuthGuard {
        dir: tempfile::TempDir,
    }

    impl TestAuthGuard {
        fn new() -> Self {
            use crate::auth_cache::{save_auth_cache, AuthCache};
            let dir = tempfile::TempDir::new().unwrap();
            let cache = AuthCache::Bearer {
                access_token: "test-token".to_string(),
                refresh_token: None,
                expires_at: i64::MAX,
            };
            save_auth_cache(dir.path(), "test", &cache).unwrap();
            TestAuthGuard { dir }
        }

        fn app_context(&self, base_url: &str) -> AppContext {
            use crate::cli::Stage;
            use std::collections::BTreeMap;
            AppContext {
                config_dir: self.dir.path().to_path_buf(),
                profile: "test".to_string(),
                stage: Stage::Dev,
                base_url: base_url.to_string(),
                defaults: BTreeMap::new(),
            }
        }

        fn app_context_with_defaults(
            &self,
            base_url: &str,
            defaults: std::collections::BTreeMap<String, String>,
        ) -> AppContext {
            use crate::cli::Stage;
            AppContext {
                config_dir: self.dir.path().to_path_buf(),
                profile: "test".to_string(),
                stage: Stage::Dev,
                base_url: base_url.to_string(),
                defaults,
            }
        }
    }

    fn clinician_response(id: &str, name: &str, email: &str, enabled: bool) -> String {
        serde_json::json!({
            "data": {
                "type": "clinicians",
                "id": id,
                "attributes": { "name": name, "email": email, "enabled": enabled }
            }
        })
        .to_string()
    }

    fn clinician_list_response(clinicians: &[(&str, &str, &str, bool)]) -> String {
        let data: Vec<serde_json::Value> = clinicians
            .iter()
            .map(|(id, name, email, enabled)| {
                serde_json::json!({
                    "type": "clinicians",
                    "id": id,
                    "attributes": { "name": name, "email": email, "enabled": enabled }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    #[tokio::test]
    async fn test_enable_by_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "11111111-1111-1111-1111-111111111111";
        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(uuid, "Alice", "alice@example.com", true))
            .create_async()
            .await;

        let out = Output { json: false };
        enable(&_auth.app_context(&server.url()), uuid, &out)
            .await
            .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_disable_by_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "22222222-2222-2222-2222-222222222222";
        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(uuid, "Bob", "bob@example.com", false))
            .create_async()
            .await;

        let out = Output { json: false };
        disable(&_auth.app_context(&server.url()), uuid, &out)
            .await
            .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_enable_by_email_looks_up_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "33333333-3333-3333-3333-333333333333";
        let email = "carol@example.com";

        let get_mock = server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[(uuid, "Carol", email, false)]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(uuid, "Carol", email, true))
            .create_async()
            .await;

        let out = Output { json: false };
        enable(&_auth.app_context(&server.url()), email, &out)
            .await
            .unwrap();

        get_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_email_not_found_returns_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = enable(
            &_auth.app_context(&server.url()),
            "missing@example.com",
            &out,
        )
        .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no clinician found with email missing@example.com"));
    }

    #[tokio::test]
    async fn test_email_lookup_is_case_insensitive() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "44444444-4444-4444-4444-444444444444";

        let _get_mock = server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[(
                uuid,
                "Dave",
                "Dave@Example.Com",
                false,
            )]))
            .create_async()
            .await;

        let _patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(uuid, "Dave", "Dave@Example.Com", true))
            .create_async()
            .await;

        let out = Output { json: false };
        enable(&_auth.app_context(&server.url()), "DAVE@EXAMPLE.COM", &out)
            .await
            .unwrap();
    }

    #[test]
    fn test_clinician_output_plain_enabled() {
        let output = ClinicianOutput {
            id: "abc-123".to_string(),
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            enabled: true,
        };
        assert_eq!(output.plain(), "Alice Smith (abc-123) is now enabled");
    }

    #[test]
    fn test_clinician_output_plain_disabled() {
        let output = ClinicianOutput {
            id: "abc-123".to_string(),
            name: "Bob Jones".to_string(),
            email: "bob@example.com".to_string(),
            enabled: false,
        };
        assert_eq!(output.plain(), "Bob Jones (abc-123) is now disabled");
    }

    #[test]
    fn test_grant_output_plain() {
        let output = GrantOutput {
            clinician_id: "11111111-1111-1111-1111-111111111111".to_string(),
            clinician_name: "Joe Smith".to_string(),
            role_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
            role_name: "admin".to_string(),
        };
        assert_eq!(
            output.plain(),
            "Joe Smith (11111111-1111-1111-1111-111111111111) granted 'admin' role"
        );
    }

    #[test]
    fn test_grant_output_json_fields() {
        let output = GrantOutput {
            clinician_id: "clin-id".to_string(),
            clinician_name: "Joe Smith".to_string(),
            role_id: "role-id".to_string(),
            role_name: "admin".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["clinician_id"], "clin-id");
        assert_eq!(json["clinician_name"], "Joe Smith");
        assert_eq!(json["role_id"], "role-id");
        assert_eq!(json["role_name"], "admin");
    }

    fn role_list_response(roles: &[(&str, &str)]) -> String {
        let data: Vec<serde_json::Value> = roles
            .iter()
            .map(|(id, name)| {
                serde_json::json!({
                    "type": "roles",
                    "id": id,
                    "attributes": { "name": name }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    #[tokio::test]
    async fn test_grant_by_uuid_and_role_name() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "55555555-5555-5555-5555-555555555555";
        let role_uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

        let roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid, "admin")]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                clinician_uuid,
                "Joe Smith",
                "joe@example.com",
                true,
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        grant(
            &_auth.app_context(&server.url()),
            clinician_uuid,
            "admin",
            &out,
        )
        .await
        .unwrap();

        roles_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_grant_by_email_and_role_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "66666666-6666-6666-6666-666666666666";
        let role_uuid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let email = "jane@example.com";

        let clinicians_mock = server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[(
                clinician_uuid,
                "Jane Doe",
                email,
                true,
            )]))
            .create_async()
            .await;

        let roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid, "editor")]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(clinician_uuid, "Jane Doe", email, true))
            .create_async()
            .await;

        let out = Output { json: false };
        grant(&_auth.app_context(&server.url()), email, role_uuid, &out)
            .await
            .unwrap();

        clinicians_mock.assert_async().await;
        roles_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_grant_role_not_found_returns_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "77777777-7777-7777-7777-777777777777";

        let _roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = grant(
            &_auth.app_context(&server.url()),
            clinician_uuid,
            "nonexistent",
            &out,
        )
        .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no role found with name 'nonexistent'"));
    }

    #[tokio::test]
    async fn test_grant_role_uuid_is_case_insensitive() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "88888888-8888-8888-8888-888888888888";
        let role_uuid_lower = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        let role_uuid_upper = "CCCCCCCC-CCCC-CCCC-CCCC-CCCCCCCCCCCC";

        let _roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid_lower, "viewer")]))
            .create_async()
            .await;

        let _patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                clinician_uuid,
                "Test User",
                "test@example.com",
                true,
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        grant(
            &_auth.app_context(&server.url()),
            clinician_uuid,
            role_uuid_upper,
            &out,
        )
        .await
        .unwrap();
    }

    #[test]
    fn test_clinician_output_json_fields() {
        let output = ClinicianOutput {
            id: "abc-123".to_string(),
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            enabled: true,
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["id"], "abc-123");
        assert_eq!(json["name"], "Alice Smith");
        assert_eq!(json["email"], "alice@example.com");
        assert_eq!(json["enabled"], true);
    }

    // --- prepare helpers ---

    fn team_list_response(teams: &[(&str, &str, &str)]) -> String {
        let data: Vec<serde_json::Value> = teams
            .iter()
            .map(|(id, name, abbr)| {
                serde_json::json!({
                    "type": "teams",
                    "id": id,
                    "attributes": { "name": name, "abbr": abbr }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    fn workspace_list_response(workspaces: &[(&str, bool)]) -> String {
        let data: Vec<serde_json::Value> = workspaces
            .iter()
            .map(|(id, default_for_clinicians)| {
                serde_json::json!({
                    "type": "workspaces",
                    "id": id,
                    "attributes": { "settings": { "default_for_clinicians": default_for_clinicians } }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    // Registers all mocks needed for a prepare call:
    // GET /clinicians/{uuid}, GET /roles, GET /teams, GET /workspaces, PATCH /clinicians/{uuid}
    // Returns the mocks so callers can assert them.
    struct PrepareMocks {
        roles_mock: mockito::Mock,
        teams_mock: mockito::Mock,
        workspaces_mock: mockito::Mock,
        patch_mock: mockito::Mock,
    }

    async fn setup_prepare_mocks_by_uuid(
        server: &mut mockito::ServerGuard,
        clinician_uuid: &str,
        clinician_name: &str,
        clinician_email: &str,
        role_uuid: &str,
        role_name: &str,
        team_uuid: &str,
        team_name: &str,
        team_abbr: &str,
        workspaces: &[(&str, bool)],
    ) -> PrepareMocks {
        server
            .mock("GET", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                clinician_uuid,
                clinician_name,
                clinician_email,
                true,
            ))
            .create_async()
            .await;

        let roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid, role_name)]))
            .create_async()
            .await;

        let teams_mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[(team_uuid, team_name, team_abbr)]))
            .create_async()
            .await;

        let workspaces_mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response(workspaces))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                clinician_uuid,
                clinician_name,
                clinician_email,
                true,
            ))
            .create_async()
            .await;

        PrepareMocks {
            roles_mock,
            teams_mock,
            workspaces_mock,
            patch_mock,
        }
    }

    #[tokio::test]
    async fn test_prepare_staff_by_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let role_uuid = "role-rw-uuid-0000-0000-000000000000";
        let team_uuid = "team-other-uuid-000-0000-000000000000";

        let mocks = setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Alice Staff",
            "alice@roundingwell.com",
            role_uuid,
            "rw",
            team_uuid,
            "Other Team",
            "OT",
            &[],
        )
        .await;

        let out = Output { json: false };
        prepare(&_auth.app_context(&server.url()), uuid, &out)
            .await
            .unwrap();

        mocks.roles_mock.assert_async().await;
        mocks.teams_mock.assert_async().await;
        mocks.workspaces_mock.assert_async().await;
        mocks.patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_prepare_employee_by_email() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let email = "bob@external.com";
        let role_uuid = "role-emp-uuid-0000-0000-000000000000";
        let team_uuid = "team-nurse-uuid-000-0000-000000000000";

        server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[(uuid, "Bob Jones", email, true)]))
            .create_async()
            .await;

        server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid, "employee")]))
            .create_async()
            .await;

        server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[(team_uuid, "Nursing", "NUR")]))
            .create_async()
            .await;

        server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response(&[]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(uuid, "Bob Jones", email, true))
            .create_async()
            .await;

        let out = Output { json: false };
        prepare(&_auth.app_context(&server.url()), email, &out)
            .await
            .unwrap();

        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_prepare_adds_to_default_workspaces() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        let role_uuid = "role-rw-uuid-1111-1111-111111111111";
        let team_uuid = "team-other-uuid-111-1111-111111111111";
        let ws1 = "ws-uuid-1111-1111-1111-111111111111";
        let ws2 = "ws-uuid-2222-2222-2222-222222222222";

        setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Carol Staff",
            "carol@roundingwell.com",
            role_uuid,
            "rw",
            team_uuid,
            "Other Team",
            "OT",
            &[(ws1, true), (ws2, true)],
        )
        .await;

        let post_mock1 = server
            .mock(
                "POST",
                format!("/workspaces/{}/relationships/clinicians", ws1).as_str(),
            )
            .with_status(204)
            .create_async()
            .await;

        let post_mock2 = server
            .mock(
                "POST",
                format!("/workspaces/{}/relationships/clinicians", ws2).as_str(),
            )
            .with_status(204)
            .create_async()
            .await;

        let out = Output { json: false };
        prepare(&_auth.app_context(&server.url()), uuid, &out)
            .await
            .unwrap();

        post_mock1.assert_async().await;
        post_mock2.assert_async().await;
    }

    #[tokio::test]
    async fn test_prepare_skips_non_default_workspaces() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "dddddddd-dddd-dddd-dddd-dddddddddddd";
        let role_uuid = "role-rw-uuid-2222-2222-222222222222";
        let team_uuid = "team-other-uuid-222-2222-222222222222";
        let ws_default = "ws-uuid-default-0000-0000-000000000000";
        let ws_non_default = "ws-uuid-nondft-0000-0000-000000000000";

        setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Dave Staff",
            "dave@roundingwell.com",
            role_uuid,
            "rw",
            team_uuid,
            "Other Team",
            "OT",
            &[(ws_default, true), (ws_non_default, false)],
        )
        .await;

        let post_mock = server
            .mock(
                "POST",
                format!("/workspaces/{}/relationships/clinicians", ws_default).as_str(),
            )
            .with_status(204)
            .create_async()
            .await;

        // This mock should NOT be called; if it is, the test will fail
        let post_non_default = server
            .mock(
                "POST",
                format!("/workspaces/{}/relationships/clinicians", ws_non_default).as_str(),
            )
            .expect(0)
            .create_async()
            .await;

        let out = Output { json: false };
        prepare(&_auth.app_context(&server.url()), uuid, &out)
            .await
            .unwrap();

        post_mock.assert_async().await;
        post_non_default.assert_async().await;
    }

    #[tokio::test]
    async fn test_prepare_no_default_workspaces() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
        let role_uuid = "role-emp-uuid-3333-3333-333333333333";
        let team_uuid = "team-nurse-uuid-333-3333-333333333333";

        setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Eve Employee",
            "eve@external.com",
            role_uuid,
            "employee",
            team_uuid,
            "Nursing",
            "NUR",
            &[],
        )
        .await;

        let out = Output { json: false };
        let result = prepare(&_auth.app_context(&server.url()), uuid, &out).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prepare_role_not_found() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "ffffffff-ffff-ffff-ffff-ffffffffffff";

        server
            .mock("GET", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                uuid,
                "Frank Staff",
                "frank@roundingwell.com",
                true,
            ))
            .create_async()
            .await;

        server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = prepare(&_auth.app_context(&server.url()), uuid, &out).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no role found with name 'rw'"));
    }

    #[tokio::test]
    async fn test_prepare_team_not_found() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "11111111-2222-3333-4444-555555555555";
        let role_uuid = "role-rw-uuid-4444-4444-444444444444";

        server
            .mock("GET", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                uuid,
                "Grace Staff",
                "grace@roundingwell.com",
                true,
            ))
            .create_async()
            .await;

        server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid, "rw")]))
            .create_async()
            .await;

        server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = prepare(&_auth.app_context(&server.url()), uuid, &out).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no team found with uuid or abbr 'OT'"));
    }

    #[test]
    fn test_prepare_output_plain_staff() {
        let output = PrepareOutput {
            clinician_id: "clin-uuid-staff".to_string(),
            clinician_name: "Alice Smith".to_string(),
            is_staff: true,
            role_id: "role-uuid".to_string(),
            role_name: "rw".to_string(),
            team_id: "team-uuid".to_string(),
            team_name: "other".to_string(),
            hidden: true,
            workspace_ids: vec!["ws-uuid-1".to_string(), "ws-uuid-2".to_string()],
        };
        assert_eq!(
            output.plain(),
            "Alice Smith (clin-uuid-staff) prepared as staff: role=rw, team=other, hidden=true, workspaces=[ws-uuid-1, ws-uuid-2]"
        );
    }

    #[test]
    fn test_prepare_output_plain_employee() {
        let output = PrepareOutput {
            clinician_id: "clin-uuid-emp".to_string(),
            clinician_name: "Bob Jones".to_string(),
            is_staff: false,
            role_id: "role-uuid".to_string(),
            role_name: "employee".to_string(),
            team_id: "team-uuid".to_string(),
            team_name: "nurse".to_string(),
            hidden: false,
            workspace_ids: vec![],
        };
        assert_eq!(
            output.plain(),
            "Bob Jones (clin-uuid-emp) prepared as employee: role=employee, team=nurse, hidden=false, workspaces=[]"
        );
    }

    #[test]
    fn test_prepare_output_json_fields() {
        let output = PrepareOutput {
            clinician_id: "clin-id".to_string(),
            clinician_name: "Alice Smith".to_string(),
            is_staff: true,
            role_id: "role-id".to_string(),
            role_name: "rw".to_string(),
            team_id: "team-id".to_string(),
            team_name: "other".to_string(),
            hidden: true,
            workspace_ids: vec!["ws-1".to_string(), "ws-2".to_string()],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["id"], "clin-id");
        assert_eq!(json["name"], "Alice Smith");
        assert_eq!(json["is_staff"], true);
        assert_eq!(json["role_id"], "role-id");
        assert_eq!(json["role_name"], "rw");
        assert_eq!(json["team_id"], "team-id");
        assert_eq!(json["team_name"], "other");
        assert_eq!(json["hidden"], true);
        assert_eq!(json["workspace_ids"][0], "ws-1");
        assert_eq!(json["workspace_ids"][1], "ws-2");
    }

    #[tokio::test]
    async fn test_prepare_workspace_failure_warns_and_continues() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
        let role_uuid = "role-rw-uuid-5555-5555-555555555555";
        let team_uuid = "team-other-uuid-555-5555-555555555555";
        let ws_ok = "ws-uuid-ok00-0000-0000-000000000000";
        let ws_fail = "ws-uuid-fail-0000-0000-000000000000";

        setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Hannah Staff",
            "hannah@roundingwell.com",
            role_uuid,
            "rw",
            team_uuid,
            "Other Team",
            "OT",
            &[(ws_ok, true), (ws_fail, true)],
        )
        .await;

        server
            .mock(
                "POST",
                format!("/workspaces/{}/relationships/clinicians", ws_ok).as_str(),
            )
            .with_status(204)
            .create_async()
            .await;

        server
            .mock(
                "POST",
                format!("/workspaces/{}/relationships/clinicians", ws_fail).as_str(),
            )
            .with_status(500)
            .with_body("internal error")
            .create_async()
            .await;

        let out = Output { json: false };
        // Command should succeed despite the workspace failure
        let result = prepare(&_auth.app_context(&server.url()), uuid, &out).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prepare_uses_config_default_for_non_staff() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "11111111-cccc-cccc-cccc-111111111111";
        let role_uuid = "role-physician-uuid-0000-000000000000";
        let team_uuid = "team-icu-uuid-00000-0000-000000000000";

        let mocks = setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Carol Clinician",
            "carol@external.com",
            role_uuid,
            "physician",
            team_uuid,
            "ICU Team",
            "ICU",
            &[],
        )
        .await;

        let defaults = [
            ("role".to_string(), "physician".to_string()),
            ("team".to_string(), "ICU".to_string()),
        ]
        .into();
        let out = Output { json: false };
        prepare(
            &_auth.app_context_with_defaults(&server.url(), defaults),
            uuid,
            &out,
        )
        .await
        .unwrap();

        mocks.roles_mock.assert_async().await;
        mocks.teams_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_prepare_falls_back_to_defaults_when_config_absent() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "22222222-cccc-cccc-cccc-222222222222";
        let role_uuid = "role-emp-uuid-2222-2222-222222222222";
        let team_uuid = "team-nur-uuid-2222-2222-222222222222";

        let mocks = setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Dave Clinician",
            "dave@external.com",
            role_uuid,
            "employee",
            team_uuid,
            "Nursing",
            "NUR",
            &[],
        )
        .await;

        let out = Output { json: false };
        // No defaults configured — should use hard-coded fallbacks
        prepare(&_auth.app_context(&server.url()), uuid, &out)
            .await
            .unwrap();

        mocks.roles_mock.assert_async().await;
        mocks.teams_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_prepare_ignores_non_staff_defaults_for_staff() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "33333333-cccc-cccc-cccc-333333333333";
        let role_uuid = "role-rw-uuid-3333-3333-333333333333";
        let team_uuid = "team-ot-uuid-33333-3333-333333333333";

        let mocks = setup_prepare_mocks_by_uuid(
            &mut server,
            uuid,
            "Eve Staff",
            "eve@roundingwell.com",
            role_uuid,
            "rw",
            team_uuid,
            "Other Team",
            "OT",
            &[],
        )
        .await;

        // Non-staff defaults set — but staff path should ignore them
        let defaults = [
            ("role".to_string(), "physician".to_string()),
            ("team".to_string(), "ICU".to_string()),
        ]
        .into();
        let out = Output { json: false };
        prepare(
            &_auth.app_context_with_defaults(&server.url(), defaults),
            uuid,
            &out,
        )
        .await
        .unwrap();

        // Staff path must use "rw"/"OT"
        mocks.roles_mock.assert_async().await;
        mocks.teams_mock.assert_async().await;
    }

    fn update_clinician_response(
        id: &str,
        name: &str,
        email: &str,
        enabled: bool,
        npi: Option<&str>,
        credentials: &[&str],
    ) -> String {
        serde_json::json!({
            "data": {
                "type": "clinicians",
                "id": id,
                "attributes": {
                    "name": name,
                    "email": email,
                    "enabled": enabled,
                    "npi": npi,
                    "credentials": credentials
                }
            }
        })
        .to_string()
    }

    // 8.1 — update by UUID
    #[tokio::test]
    async fn test_update_by_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": {
                    "type": "clinicians",
                    "id": uuid,
                    "attributes": { "name": "Jane Doe" }
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Jane Doe",
                "jane@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        update(
            &_auth.app_context(&server.url()),
            uuid,
            "name",
            Some("Jane Doe"),
            &out,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    // 8.1b — update sends correct field name as JSON key (not literal "field")
    #[tokio::test]
    async fn test_update_sends_correct_field_key() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "data": {
                    "type": "clinicians",
                    "id": uuid,
                    "attributes": { "email": "new@example.com" }
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Jane Doe",
                "new@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        update(
            &_auth.app_context(&server.url()),
            uuid,
            "email",
            Some("new@example.com"),
            &out,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    // 8.2 — update by email
    #[tokio::test]
    async fn test_update_by_email() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let email = "jane@example.com";

        let get_mock = server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[(uuid, "Jane", email, true)]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Jane",
                "jane2@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        update(
            &_auth.app_context(&server.url()),
            email,
            "email",
            Some("jane2@example.com"),
            &out,
        )
        .await
        .unwrap();

        get_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    // 8.3 — update with target "me"
    #[tokio::test]
    async fn test_update_by_me() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "cccccccc-cccc-cccc-cccc-cccccccccccc";

        let me_mock = server
            .mock("GET", "/clinicians/me")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Me User",
                "me@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "New Name",
                "me@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        update(
            &_auth.app_context(&server.url()),
            "me",
            "name",
            Some("New Name"),
            &out,
        )
        .await
        .unwrap();

        me_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    // 8.4 — validation rejects empty name
    #[test]
    fn test_validate_field_rejects_empty_name() {
        let result = validate_field("name", Some(""));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid name"));
    }

    // 8.5 — validation rejects invalid email
    #[test]
    fn test_validate_field_rejects_invalid_email() {
        let result = validate_field("email", Some("not-an-email"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid email"));
    }

    // 8.6 — validation rejects non-10-digit NPI
    #[test]
    fn test_validate_field_rejects_invalid_npi() {
        // too short
        let result = validate_field("npi", Some("12345"));
        assert!(result.is_err(), "expected error for too-short NPI");
        // too long
        let result = validate_field("npi", Some("12345678901"));
        assert!(result.is_err(), "expected error for too-long NPI");
        // non-numeric
        let result = validate_field("npi", Some("123456789a"));
        assert!(result.is_err(), "expected error for non-numeric NPI");
    }

    // 8.7 — omitted/empty value with --field npi sends null
    #[tokio::test]
    async fn test_update_npi_null_when_empty() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "dddddddd-dddd-dddd-dddd-dddddddddddd";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": {
                    "attributes": { "npi": serde_json::Value::Null }
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Doc",
                "doc@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        // omitted value (None)
        update(&_auth.app_context(&server.url()), uuid, "npi", None, &out)
            .await
            .unwrap();

        mock.assert_async().await;
    }

    // 8.8 — omitted/empty value with --field credentials sends []
    #[tokio::test]
    async fn test_update_credentials_empty_sends_array() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": {
                    "attributes": { "credentials": [] }
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Doc",
                "doc@example.com",
                true,
                None,
                &[],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        update(
            &_auth.app_context(&server.url()),
            uuid,
            "credentials",
            None,
            &out,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    // 8.9 — credentials split on comma
    #[tokio::test]
    async fn test_update_credentials_split_on_comma() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "ffffffff-ffff-ffff-ffff-ffffffffffff";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": {
                    "attributes": { "credentials": ["RN", "MD"] }
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(update_clinician_response(
                uuid,
                "Doc",
                "doc@example.com",
                true,
                None,
                &["RN", "MD"],
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        update(
            &_auth.app_context(&server.url()),
            uuid,
            "credentials",
            Some("RN,MD"),
            &out,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    // 8.10 — unsupported field name returns error
    #[test]
    fn test_validate_field_rejects_unknown_field() {
        let result = validate_field("unknown_field", Some("value"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("unsupported field"),
            "expected 'unsupported field' in: {}",
            err
        );
        assert!(
            err.contains("name") && err.contains("email"),
            "expected allowed fields listed in: {}",
            err
        );
    }

    #[test]
    fn test_assign_team_output_plain() {
        let output = AssignTeamOutput {
            clinician_id: "11111111-1111-1111-1111-111111111111".to_string(),
            clinician_name: "Joe Smith".to_string(),
            team_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
            team_name: "Nursing".to_string(),
        };
        assert_eq!(
            output.plain(),
            "Joe Smith (11111111-1111-1111-1111-111111111111) assigned to 'Nursing' team"
        );
    }

    #[tokio::test]
    async fn test_assign_by_uuid_and_team_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "aaaaaaaa-0000-0000-0000-000000000002";
        let team_uuid = "bbbbbbbb-0000-0000-0000-000000000002";

        let teams_mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[(team_uuid, "Nursing", "NUR")]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                clinician_uuid,
                "Bob",
                "bob@example.com",
                true,
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        assign(
            &_auth.app_context(&server.url()),
            clinician_uuid,
            team_uuid,
            &out,
        )
        .await
        .unwrap();

        teams_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_assign_by_email_and_team_abbr() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "aaaaaaaa-0000-0000-0000-000000000003";
        let team_uuid = "bbbbbbbb-0000-0000-0000-000000000003";
        let email = "carol@example.com";

        let clinicians_mock = server
            .mock("GET", "/clinicians")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_list_response(&[(
                clinician_uuid,
                "Carol",
                email,
                true,
            )]))
            .create_async()
            .await;

        let teams_mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[(team_uuid, "Nursing", "NUR")]))
            .create_async()
            .await;

        let patch_mock = server
            .mock("PATCH", format!("/clinicians/{}", clinician_uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(clinician_uuid, "Carol", email, true))
            .create_async()
            .await;

        let out = Output { json: false };
        assign(&_auth.app_context(&server.url()), email, "nur", &out)
            .await
            .unwrap();

        clinicians_mock.assert_async().await;
        teams_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_assign_team_not_found_returns_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let clinician_uuid = "aaaaaaaa-0000-0000-0000-000000000004";

        let _teams_mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = assign(
            &_auth.app_context(&server.url()),
            clinician_uuid,
            "nonexistent",
            &out,
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no team found with uuid or abbr 'nonexistent'"));
    }

    #[tokio::test]
    async fn test_update_api_error_surfaced() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "11111111-2222-3333-4444-555555555555";

        let _mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .with_status(422)
            .with_body(r#"{"errors":[{"detail":"invalid"}]}"#)
            .create_async()
            .await;

        let out = Output { json: false };
        let result = update(
            &_auth.app_context(&server.url()),
            uuid,
            "name",
            Some("Jane"),
            &out,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("422"), "expected 422 status in: {}", err);
    }

    // --- register tests ---

    #[tokio::test]
    async fn test_register_success_no_role_no_team() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";

        let post_mock = server
            .mock("POST", "/clinicians")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                uuid,
                "Jane Doe",
                "jane@example.com",
                true,
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        register(
            &_auth.app_context(&server.url()),
            "jane@example.com",
            "Jane Doe",
            None,
            None,
            &out,
        )
        .await
        .unwrap();

        post_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_register_with_role() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "ffffffff-ffff-ffff-ffff-ffffffffffff";
        let role_uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

        let roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[(role_uuid, "Staff")]))
            .create_async()
            .await;

        let post_mock = server
            .mock("POST", "/clinicians")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                uuid,
                "Jane Doe",
                "jane@example.com",
                true,
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        register(
            &_auth.app_context(&server.url()),
            "jane@example.com",
            "Jane Doe",
            Some("Staff"),
            None,
            &out,
        )
        .await
        .unwrap();

        roles_mock.assert_async().await;
        post_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_register_with_team() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "11111111-aaaa-aaaa-aaaa-111111111111";
        let team_uuid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

        let teams_mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[(team_uuid, "ICU", "ICU")]))
            .create_async()
            .await;

        let post_mock = server
            .mock("POST", "/clinicians")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(clinician_response(
                uuid,
                "Jane Doe",
                "jane@example.com",
                true,
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        register(
            &_auth.app_context(&server.url()),
            "jane@example.com",
            "Jane Doe",
            None,
            Some("ICU"),
            &out,
        )
        .await
        .unwrap();

        teams_mock.assert_async().await;
        post_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_register_blank_name_returns_error_without_network() {
        let _auth = TestAuthGuard::new();
        let out = Output { json: false };
        let result = register(
            &_auth.app_context("http://unused"),
            "jane@example.com",
            "   ",
            None,
            None,
            &out,
        )
        .await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("invalid name"),
            "expected name validation error"
        );
    }

    #[tokio::test]
    async fn test_register_invalid_email_returns_error_without_network() {
        let _auth = TestAuthGuard::new();
        let out = Output { json: false };
        let result = register(
            &_auth.app_context("http://unused"),
            "not-an-email",
            "Jane Doe",
            None,
            None,
            &out,
        )
        .await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("invalid email"),
            "expected email validation error"
        );
    }

    #[tokio::test]
    async fn test_register_invalid_role_returns_error_without_post() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let roles_mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = register(
            &_auth.app_context(&server.url()),
            "jane@example.com",
            "Jane Doe",
            Some("nonexistent-role"),
            None,
            &out,
        )
        .await;

        assert!(result.is_err());
        roles_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_register_api_error_surfaced() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let post_mock = server
            .mock("POST", "/clinicians")
            .with_status(422)
            .with_body(r#"{"errors":[{"detail":"email already taken"}]}"#)
            .create_async()
            .await;

        let out = Output { json: false };
        let result = register(
            &_auth.app_context(&server.url()),
            "jane@example.com",
            "Jane Doe",
            None,
            None,
            &out,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("422"), "expected 422 in: {}", err);
        post_mock.assert_async().await;
    }
}
