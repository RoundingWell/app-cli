//! `rw clinicians show <target>` — shows a clinician by UUID, email, or "me".

use anyhow::Result;
use uuid::Uuid;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::output::Output;

use super::client::{fetch_clinician_by_email_filter, fetch_clinician_by_uuid, fetch_clinician_me};
use super::output::ClinicianShowOutput;

pub async fn show(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;

    let clinician = if target == "me" {
        fetch_clinician_me(&api).await?
    } else if Uuid::parse_str(target).is_ok() {
        fetch_clinician_by_uuid(&api, target).await?
    } else {
        fetch_clinician_by_email_filter(&api, target).await?
    };

    out.print(&ClinicianShowOutput {
        id: clinician.id,
        name: clinician.attributes.name,
        email: clinician.attributes.email,
        enabled: clinician.attributes.enabled,
        npi: clinician.attributes.npi,
        credentials: clinician.attributes.credentials,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_show_by_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

        let mock = server
            .mock("GET", format!("/clinicians/{}", uuid).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": {
                        "type": "clinicians",
                        "id": uuid,
                        "attributes": {
                            "name": "Alice Show",
                            "email": "alice@example.com",
                            "enabled": true,
                            "npi": "1234567890",
                            "credentials": ["MD"]
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), uuid, &out).await;
        assert!(result.is_ok(), "expected ok, got {:?}", result);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_by_email() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let email = "bob@example.com";

        let mock = server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/clinicians\?".to_string()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": [{
                        "type": "clinicians",
                        "id": uuid,
                        "attributes": {
                            "name": "Bob Show",
                            "email": email,
                            "enabled": true,
                            "npi": null,
                            "credentials": []
                        }
                    }]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), email, &out).await;
        assert!(result.is_ok(), "expected ok, got {:?}", result);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_me() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "cccccccc-cccc-cccc-cccc-cccccccccccc";

        let mock = server
            .mock("GET", "/clinicians/me")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": {
                        "type": "clinicians",
                        "id": uuid,
                        "attributes": {
                            "name": "Carol Me",
                            "email": "carol@roundingwell.com",
                            "enabled": true,
                            "npi": null,
                            "credentials": []
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "me", &out).await;
        assert!(result.is_ok(), "expected ok, got {:?}", result);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_by_email_not_found() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let email = "nobody@example.com";

        let mock = server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/clinicians\?".to_string()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "data": [] }).to_string())
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), email, &out).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains(email),
            "expected email in error message: {}",
            err
        );
        mock.assert_async().await;
    }
}
