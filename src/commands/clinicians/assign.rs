//! `rw clinicians assign <target> <team>` — assigns a clinician to a team.

use anyhow::Result;
use uuid::Uuid;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::Single;
use crate::output::Output;

use super::client::{resolve_team, resolve_uuid_by_email};
use super::data::ClinicianAttributes;
use super::output::AssignTeamOutput;

pub async fn assign(ctx: &AppContext, target: &str, team_target: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let clinician_uuid = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        resolve_uuid_by_email(&api, target).await?
    };
    let (team_id, team_name) = resolve_team(&api, team_target).await?;
    let body = serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": &clinician_uuid,
            "relationships": {
                "team": { "data": { "type": "teams", "id": &team_id } }
            }
        }
    });
    let resp: Single<ClinicianAttributes> = api
        .patch(&format!("clinicians/{}", clinician_uuid), &body)
        .await?;
    out.print(&AssignTeamOutput {
        clinician_id: resp.data.id,
        clinician_name: resp.data.attributes.name,
        team_id,
        team_name,
    });
    Ok(())
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
