use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

// --- JSON:API deserialization types ---

#[derive(Debug, Deserialize)]
pub(crate) struct TeamAttributes {
    pub(crate) name: String,
    pub(crate) abbr: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TeamResource {
    pub(crate) id: String,
    pub(crate) attributes: TeamAttributes,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TeamListResponse {
    pub(crate) data: Vec<TeamResource>,
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

// --- Public command functions ---

pub async fn list(ctx: &AppContext, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    let url = format!("{}/teams", ctx.base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, &auth_header)
        .send()
        .await
        .context("GET /teams failed")?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        anyhow::bail!("API returned {}: {}", status, body);
    }

    let list: TeamListResponse =
        serde_json::from_str(&body).context("failed to parse teams response")?;

    let mut teams: Vec<TeamRow> = list
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
                stage: Stage::Dev,
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
}
