use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cli::{TeamsArgs, TeamsCommands};
use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::List;
use crate::output::{CommandOutput, Output};

pub async fn dispatch(args: TeamsArgs, ctx: &AppContext, out: &Output) -> Result<()> {
    match args.command {
        TeamsCommands::List(_) => list(ctx, out).await,
        TeamsCommands::Show(a) => show(ctx, &a.target, out).await,
    }
}

// --- JSON:API attributes ---

#[derive(Debug, Deserialize)]
pub(crate) struct TeamAttributes {
    pub(crate) name: String,
    pub(crate) abbr: String,
}

// --- Output types ---

#[derive(Debug, tabled::Tabled, Serialize)]
pub struct TeamRow {
    pub id: String,
    pub abbr: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct TeamListOutput {
    pub teams: Vec<TeamRow>,
}

impl CommandOutput for TeamListOutput {
    fn plain(&self) -> String {
        use tabled::settings::Style;
        use tabled::Table;
        Table::new(&self.teams).with(Style::markdown()).to_string()
    }
}

// --- Show output types ---

#[derive(Debug, Serialize)]
pub struct TeamShowOutput {
    pub id: String,
    pub abbr: String,
    pub name: String,
}

impl CommandOutput for TeamShowOutput {
    fn plain(&self) -> String {
        format!(
            "ID:   {}\nAbbr: {}\nName: {}",
            self.id, self.abbr, self.name
        )
    }
}

// --- Public command functions ---

pub async fn list(ctx: &AppContext, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<TeamAttributes> = api.get("teams").await?;

    let mut teams: Vec<TeamRow> = resp
        .data
        .into_iter()
        .map(|t| TeamRow {
            id: t.id,
            abbr: t.attributes.abbr,
            name: t.attributes.name,
        })
        .collect();

    teams.sort_by(|a, b| a.abbr.cmp(&b.abbr));

    out.print(&TeamListOutput { teams });
    Ok(())
}

pub async fn show(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<TeamAttributes> = api.get("teams").await?;
    let target_lower = target.to_lowercase();

    let team = if Uuid::parse_str(target).is_ok() {
        resp.data
            .into_iter()
            .find(|t| t.id.to_lowercase() == target_lower)
            .ok_or_else(|| anyhow::anyhow!("no team found with id {}", target))?
    } else {
        resp.data
            .into_iter()
            .find(|t| t.attributes.abbr.to_lowercase() == target_lower)
            .ok_or_else(|| anyhow::anyhow!("no team found with abbr '{}'", target))?
    };

    out.print(&TeamShowOutput {
        id: team.id,
        abbr: team.attributes.abbr,
        name: team.attributes.name,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

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
                auth_profile: "test".to_string(),
                stage: Stage::Dev,
                auth_stage: Stage::Dev,
                base_url: base_url.to_string(),
                defaults: BTreeMap::new(),
            }
        }
    }

    fn team_list_response(teams: &[(&str, &str, &str)]) -> String {
        let data: Vec<serde_json::Value> = teams
            .iter()
            .map(|(id, abbr, name)| {
                serde_json::json!({
                    "type": "teams",
                    "id": id,
                    "attributes": { "abbr": abbr, "name": name }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    #[tokio::test]
    async fn test_list_multiple_teams_sorted_by_abbr() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[
                ("uuid-b", "NUR", "Nursing"),
                ("uuid-a", "ADM", "Administration"),
                ("uuid-c", "PHY", "Physician"),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_empty_teams() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_api_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }

    // --- show tests ---

    #[tokio::test]
    async fn test_show_by_uuid_plain() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[
                (
                    "aaaaaaaa-0000-0000-0000-000000000001",
                    "ADM",
                    "Administration",
                ),
                ("aaaaaaaa-0000-0000-0000-000000000002", "NUR", "Nursing"),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(
            &_auth.app_context(&server.url()),
            "aaaaaaaa-0000-0000-0000-000000000001",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_by_abbr_plain() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[
                ("uuid-1", "ADM", "Administration"),
                ("uuid-2", "NUR", "Nursing"),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "NUR", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_json_output() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[("uuid-1", "ADM", "Administration")]))
            .create_async()
            .await;

        let out = Output { json: true };
        let result = show(&_auth.app_context(&server.url()), "ADM", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_target_not_found() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(team_list_response(&[("uuid-1", "ADM", "Administration")]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "no-such-team", &out).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no team found"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_api_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/teams")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "ADM", &out).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }
}
