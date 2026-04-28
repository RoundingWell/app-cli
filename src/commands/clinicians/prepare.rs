//! `rw clinicians prepare <target>` — sets the clinician's role, team, hidden
//! flag, and default workspace memberships based on whether the email is a
//! `@roundingwell.com` staff address or external.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

use crate::config::AppContext;
use crate::output::Output;

use super::client::{apply_auth, fetch_clinician_by_email, fetch_clinician_by_uuid, resolve_team};
use super::data::{ClinicianResource, ClinicianSingleResponse};
use super::output::PrepareOutput;

// --- prepare-only deserialization types for `/workspaces` ---

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

pub async fn prepare(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    let auth_header = crate::commands::auth::require_auth(ctx).await?;
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
    let (role_id, role_name) = crate::commands::roles::resolve_role(
        &client,
        &ctx.base_url,
        &auth_header,
        role_name_target,
    )
    .await?;

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
            "attributes": { "hidden": hidden },
            "relationships": {
                "role": { "data": { "type": "roles", "id": role_uuid } },
                "team": { "data": { "type": "teams", "id": team_uuid } }
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
        "data": [{ "type": "clinicians", "id": clinician_uuid }]
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
    use super::super::output::PrepareOutput;
    use super::super::testing::*;
    use super::*;
    use crate::output::CommandOutput;
    use mockito::Server;

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

        mocks.roles_mock.assert_async().await;
        mocks.teams_mock.assert_async().await;
    }
}
