//! `rw clinicians update <target> --field <field> [--value <value>]`.

use anyhow::{bail, Context, Result};
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;
use uuid::Uuid;
use validator::Validate;

use crate::config::AppContext;
use crate::output::Output;

use super::client::{apply_auth, resolve_me, resolve_uuid_by_email};
use super::data::ClinicianSingleResponse;
use super::output::ClinicianUpdateOutput;

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

pub(super) fn validate_field(field: &str, value: Option<&str>) -> Result<()> {
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

// --- Public command ---

pub async fn update(
    ctx: &AppContext,
    target: &str,
    field: &str,
    value: Option<&str>,
    out: &Output,
) -> Result<()> {
    validate_field(field, value)?;

    let auth_header = crate::commands::auth::require_auth(ctx).await?;
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
            "attributes": { (field): attr_value }
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

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use mockito::Server;

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

    #[test]
    fn test_validate_field_rejects_empty_name() {
        let result = validate_field("name", Some(""));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid name"));
    }

    #[test]
    fn test_validate_field_rejects_invalid_email() {
        let result = validate_field("email", Some("not-an-email"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid email"));
    }

    #[test]
    fn test_validate_field_rejects_invalid_npi() {
        let result = validate_field("npi", Some("12345"));
        assert!(result.is_err(), "expected error for too-short NPI");
        let result = validate_field("npi", Some("12345678901"));
        assert!(result.is_err(), "expected error for too-long NPI");
        let result = validate_field("npi", Some("123456789a"));
        assert!(result.is_err(), "expected error for non-numeric NPI");
    }

    #[tokio::test]
    async fn test_update_npi_null_when_empty() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "dddddddd-dddd-dddd-dddd-dddddddddddd";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": { "attributes": { "npi": serde_json::Value::Null } }
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
        update(&_auth.app_context(&server.url()), uuid, "npi", None, &out)
            .await
            .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_update_credentials_empty_sends_array() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": { "attributes": { "credentials": [] } }
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

    #[tokio::test]
    async fn test_update_credentials_split_on_comma() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let uuid = "ffffffff-ffff-ffff-ffff-ffffffffffff";

        let mock = server
            .mock("PATCH", format!("/clinicians/{}", uuid).as_str())
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "data": { "attributes": { "credentials": ["RN", "MD"] } }
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
}
