//! `rw clinicians register <email> <name>` — registers a new clinician.

use anyhow::Result;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::Single;
use crate::output::Output;

use super::client::resolve_team;
use super::data::ClinicianAttributes;
use super::output::ClinicianRegisterOutput;
use super::update::validate_field;

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

    let api = ApiClient::new(ctx).await?;

    // Resolve role and team before POST
    let role = if let Some(rt) = role_target {
        Some(crate::commands::roles::resolve_role(&api, rt).await?)
    } else {
        None
    };
    let team = if let Some(tt) = team_target {
        Some(resolve_team(&api, tt).await?)
    } else {
        None
    };

    // Build JSON:API POST body
    let mut data = serde_json::json!({
        "type": "clinicians",
        "attributes": { "email": email, "name": name }
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
    let resp: Single<ClinicianAttributes> = api.post("clinicians", &body).await?;

    out.print(&ClinicianRegisterOutput {
        id: resp.data.id,
        name: resp.data.attributes.name,
        email: resp.data.attributes.email,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use mockito::Server;

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
