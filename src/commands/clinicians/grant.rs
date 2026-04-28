//! `rw clinicians grant <target> <role>` — grants a role to a clinician.

use anyhow::Result;
use uuid::Uuid;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::Single;
use crate::output::Output;

use super::client::resolve_uuid_by_email;
use super::data::ClinicianAttributes;
use super::output::GrantOutput;

pub async fn grant(ctx: &AppContext, target: &str, role_target: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let clinician_uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&api, target).await?
    };
    let (role_id, role_name) = crate::commands::roles::resolve_role(&api, role_target).await?;
    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": &clinician_uuid,
            "relationships": {
                "role": { "data": { "type": "roles", "id": &role_id } }
            }
        }
    });
    let resp: Single<ClinicianAttributes> = api
        .patch(&format!("clinicians/{}", clinician_uuid), &body)
        .await?;
    out.print(&GrantOutput {
        clinician_id: resp.data.id,
        clinician_name: resp.data.attributes.name,
        role_id,
        role_name,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::output::GrantOutput;
    use super::super::testing::*;
    use super::*;
    use crate::output::CommandOutput;
    use mockito::Server;

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
}
