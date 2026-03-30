use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cli::Stage;
use crate::output::{CommandOutput, Output};

// --- JSON:API deserialization types ---

#[derive(Debug, Deserialize)]
struct ClinicianAttributes {
    name: String,
    email: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct RoleAttributes {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RoleResource {
    id: String,
    attributes: RoleAttributes,
}

#[derive(Debug, Deserialize)]
struct RoleListResponse {
    data: Vec<RoleResource>,
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

// --- Output types ---

#[derive(Debug, Serialize)]
pub struct AssignOutput {
    pub clinician_id: String,
    pub clinician_name: String,
    pub role_id: String,
    pub role_name: String,
}

impl CommandOutput for AssignOutput {
    fn plain(&self) -> String {
        format!(
            "{} ({}) assigned to '{}' role",
            self.clinician_name, self.clinician_id, self.role_name
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

// --- Public command functions ---

pub async fn assign(
    base_url: &str,
    organization: &str,
    stage: &Stage,
    target: &str,
    role_target: &str,
    out: &Output,
) -> Result<()> {
    let auth_header = require_auth(organization, stage).await?;
    let client = Client::new();
    let clinician_uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&client, base_url, &auth_header, target).await?
    };
    let (role_id, role_name) = resolve_role(&client, base_url, &auth_header, role_target).await?;
    let (clinician_id, clinician_name) =
        patch_clinician_role(&client, base_url, &auth_header, &clinician_uuid, &role_id).await?;
    out.print(&AssignOutput {
        clinician_id,
        clinician_name,
        role_id,
        role_name,
    });
    Ok(())
}

pub async fn enable(
    base_url: &str,
    organization: &str,
    stage: &Stage,
    target: &str,
    out: &Output,
) -> Result<()> {
    set_enabled(base_url, organization, stage, target, true, out).await
}

pub async fn disable(
    base_url: &str,
    organization: &str,
    stage: &Stage,
    target: &str,
    out: &Output,
) -> Result<()> {
    set_enabled(base_url, organization, stage, target, false, out).await
}

// --- Private helpers ---

async fn set_enabled(
    base_url: &str,
    organization: &str,
    stage: &Stage,
    target: &str,
    enabled: bool,
    out: &Output,
) -> Result<()> {
    let auth_header = require_auth(organization, stage).await?;
    let client = Client::new();
    let uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&client, base_url, &auth_header, target).await?
    };
    let result = patch_clinician(&client, base_url, &auth_header, &uuid, enabled).await?;
    out.print(&result);
    Ok(())
}

/// Resolves auth credentials, returning the `Authorization` header value.
/// Fails with a friendly message if no credentials are stored.
async fn require_auth(organization: &str, stage: &Stage) -> Result<String> {
    super::auth::resolve_auth(organization, stage)
        .await?
        .map(|a| super::auth::auth_header_value(&a))
        .ok_or_else(|| anyhow::anyhow!("not authenticated – run `rw auth login` first"))
}

fn apply_auth(req: reqwest::RequestBuilder, auth_header: &str) -> reqwest::RequestBuilder {
    req.header(reqwest::header::AUTHORIZATION, auth_header)
}

async fn resolve_uuid_by_email(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    email: &str,
) -> Result<String> {
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
        .map(|c| c.id)
        .ok_or_else(|| anyhow::anyhow!("no clinician found with email {}", email))
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

    let list: RoleListResponse =
        serde_json::from_str(&body).context("failed to parse roles response")?;

    if Uuid::parse_str(role_target).is_ok() {
        let target_lower = role_target.to_lowercase();
        list.data
            .into_iter()
            .find(|r| r.id.to_lowercase() == target_lower)
            .map(|r| (r.id, r.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no role found with id {}", role_target))
    } else {
        let target_lower = role_target.to_lowercase();
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    /// Writes a fake Bearer token to the testorg-dev auth cache so that `require_auth`
    /// succeeds. Deletes the file on drop. Safe with parallel tests: the public functions
    /// resolve auth once and hold it in memory, so a concurrent drop doesn't affect them.
    struct TestAuthGuard;

    impl TestAuthGuard {
        fn new() -> Self {
            use crate::auth_cache::{save_auth_cache, AuthCache};
            let cache = AuthCache::Bearer {
                access_token: "test-token".to_string(),
                refresh_token: None,
                expires_at: i64::MAX,
            };
            save_auth_cache("testorg", &Stage::Dev, &cache).unwrap();
            TestAuthGuard
        }
    }

    impl Drop for TestAuthGuard {
        fn drop(&mut self) {
            let _ = crate::auth_cache::delete_auth_cache("testorg", &Stage::Dev);
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
        enable(&server.url(), "testorg", &Stage::Dev, uuid, &out)
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
        disable(&server.url(), "testorg", &Stage::Dev, uuid, &out)
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
        enable(&server.url(), "testorg", &Stage::Dev, email, &out)
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
            &server.url(),
            "testorg",
            &Stage::Dev,
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
        enable(
            &server.url(),
            "testorg",
            &Stage::Dev,
            "DAVE@EXAMPLE.COM",
            &out,
        )
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
    fn test_assign_output_plain() {
        let output = AssignOutput {
            clinician_id: "11111111-1111-1111-1111-111111111111".to_string(),
            clinician_name: "Joe Smith".to_string(),
            role_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
            role_name: "admin".to_string(),
        };
        assert_eq!(
            output.plain(),
            "Joe Smith (11111111-1111-1111-1111-111111111111) assigned to 'admin' role"
        );
    }

    #[test]
    fn test_assign_output_json_fields() {
        let output = AssignOutput {
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
    async fn test_assign_by_uuid_and_role_name() {
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
        assign(
            &server.url(),
            "testorg",
            &Stage::Dev,
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
    async fn test_assign_by_email_and_role_uuid() {
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
        assign(
            &server.url(),
            "testorg",
            &Stage::Dev,
            email,
            role_uuid,
            &out,
        )
        .await
        .unwrap();

        clinicians_mock.assert_async().await;
        roles_mock.assert_async().await;
        patch_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_assign_role_not_found_returns_error() {
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
        let result = assign(
            &server.url(),
            "testorg",
            &Stage::Dev,
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
    async fn test_assign_role_uuid_is_case_insensitive() {
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
        assign(
            &server.url(),
            "testorg",
            &Stage::Dev,
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
}
