use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

// --- JSON:API deserialization types ---

#[derive(Debug, Deserialize)]
pub(crate) struct RoleAttributes {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) label: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoleResource {
    pub(crate) id: String,
    pub(crate) attributes: RoleAttributes,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoleListResponse {
    pub(crate) data: Vec<RoleResource>,
}

// --- Output types ---

#[derive(Debug, tabled::Tabled, Serialize)]
pub struct RoleRow {
    pub id: String,
    pub name: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct RoleListOutput {
    pub roles: Vec<RoleRow>,
}

impl CommandOutput for RoleListOutput {
    fn plain(&self) -> String {
        use tabled::settings::Style;
        use tabled::Table;
        Table::new(&self.roles).with(Style::markdown()).to_string()
    }
}

// --- Public command functions ---

pub async fn list(ctx: &AppContext, out: &Output) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    let url = format!("{}/roles", ctx.base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, &auth_header)
        .send()
        .await
        .context("GET /roles failed")?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        anyhow::bail!("API returned {}: {}", status, body);
    }

    let list: RoleListResponse =
        serde_json::from_str(&body).context("failed to parse roles response")?;

    let mut roles: Vec<RoleRow> = list
        .data
        .into_iter()
        .map(|r| RoleRow {
            id: r.id,
            name: r.attributes.name,
            label: r.attributes.label,
        })
        .collect();

    roles.sort_by(|a, b| a.label.cmp(&b.label));

    out.print(&RoleListOutput { roles });
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

    fn role_list_response(roles: &[(&str, &str, &str)]) -> String {
        let data: Vec<serde_json::Value> = roles
            .iter()
            .map(|(id, name, label)| {
                serde_json::json!({
                    "type": "roles",
                    "id": id,
                    "attributes": { "name": name, "label": label }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    #[tokio::test]
    async fn test_list_multiple_roles_sorted_by_label() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[
                ("uuid-b", "nurse", "Nurse"),
                ("uuid-a", "admin", "Administrator"),
                ("uuid-c", "physician", "Physician"),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&_auth.app_context(&server.url()), &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_empty_roles() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response(&[]))
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
            .mock("GET", "/roles")
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
