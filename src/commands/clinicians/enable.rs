//! `rw clinicians enable` / `disable` — toggles the `enabled` attribute.

use anyhow::Result;
use uuid::Uuid;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::Single;
use crate::output::Output;

use super::client::resolve_uuid_by_email;
use super::data::ClinicianAttributes;
use super::output::ClinicianOutput;

pub async fn enable(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    set_enabled(ctx, target, true, out).await
}

pub async fn disable(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    set_enabled(ctx, target, false, out).await
}

async fn set_enabled(ctx: &AppContext, target: &str, enabled: bool, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&api, target).await?
    };
    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": &uuid,
            "attributes": { "enabled": enabled }
        }
    });
    let resp: Single<ClinicianAttributes> =
        api.patch(&format!("clinicians/{}", uuid), &body).await?;
    out.print(&ClinicianOutput {
        id: resp.data.id,
        name: resp.data.attributes.name,
        email: resp.data.attributes.email,
        enabled: resp.data.attributes.enabled,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::output::ClinicianOutput;
    use super::super::testing::*;
    use super::*;
    use crate::output::CommandOutput;
    use mockito::Server;

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
}
