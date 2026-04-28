//! `rw clinicians assign <target> <team>` — assigns a clinician to a team.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use uuid::Uuid;

use crate::config::AppContext;
use crate::output::Output;

use super::client::{apply_auth, resolve_team, resolve_uuid_by_email};
use super::data::ClinicianSingleResponse;
use super::output::AssignTeamOutput;

pub async fn assign(ctx: &AppContext, target: &str, team_target: &str, out: &Output) -> Result<()> {
    let auth_header = crate::commands::auth::require_auth(ctx).await?;
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

    Ok((response.data.id, response.data.attributes.name))
}

#[cfg(test)]
mod tests {
    use super::super::output::AssignTeamOutput;
    use super::super::testing::*;
    use super::*;
    use crate::output::CommandOutput;
    use mockito::Server;

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
}
