use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

// --- JSON:API deserialization types ---

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceAttributes {
    pub(crate) slug: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceResource {
    pub(crate) id: String,
    pub(crate) attributes: WorkspaceAttributes,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceListResponse {
    pub(crate) data: Vec<WorkspaceResource>,
}

// --- Output types ---

#[derive(Debug, tabled::Tabled, Serialize)]
pub struct WorkspaceRow {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct WorkspaceListOutput {
    pub workspaces: Vec<WorkspaceRow>,
}

impl CommandOutput for WorkspaceListOutput {
    fn plain(&self) -> String {
        use tabled::settings::Style;
        use tabled::Table;
        Table::new(&self.workspaces)
            .with(Style::markdown())
            .to_string()
    }
}

// --- Public command functions ---

pub async fn list(ctx: &AppContext, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    let url = format!("{}/workspaces", ctx.base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, &auth_header)
        .send()
        .await
        .context("GET /workspaces failed")?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        anyhow::bail!("API returned {}: {}", status, body);
    }

    let list: WorkspaceListResponse =
        serde_json::from_str(&body).context("failed to parse workspaces response")?;

    let mut workspaces: Vec<WorkspaceRow> = list
        .data
        .into_iter()
        .map(|w| WorkspaceRow {
            id: w.id,
            slug: w.attributes.slug,
            name: w.attributes.name,
        })
        .collect();

    workspaces.sort_by(|a, b| a.name.cmp(&b.name));

    out.print(&WorkspaceListOutput { workspaces });
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

    fn workspace_list_response(workspaces: &[(&str, &str, &str)]) -> String {
        let data: Vec<serde_json::Value> = workspaces
            .iter()
            .map(|(id, slug, name)| {
                serde_json::json!({
                    "type": "workspaces",
                    "id": id,
                    "attributes": { "slug": slug, "name": name }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    #[tokio::test]
    async fn test_list_multiple_workspaces_sorted_by_name() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response(&[
                ("uuid-b", "cardiology", "Cardiology"),
                ("uuid-a", "admin", "Administration"),
                ("uuid-c", "nursing", "Nursing"),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_empty_workspaces() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_json_output() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response(&[("uuid-1", "main", "Main")]))
            .create_async()
            .await;

        let out = Output { json: true };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_api_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
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
