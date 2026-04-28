use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::List;
use crate::output::{CommandOutput, Output};

// --- JSON:API attributes ---

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceAttributes {
    pub(crate) slug: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) settings: serde_json::Map<String, serde_json::Value>,
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

// --- Show output types ---

#[derive(Debug, tabled::Tabled)]
struct SettingRow {
    name: String,
    value: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceShowOutput {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub settings: serde_json::Map<String, serde_json::Value>,
}

impl CommandOutput for WorkspaceShowOutput {
    fn plain(&self) -> String {
        use tabled::settings::Style;
        use tabled::Table;

        let mut rows: Vec<SettingRow> = self
            .settings
            .iter()
            .map(|(k, v)| SettingRow {
                name: k.clone(),
                value: v.to_string(),
            })
            .collect();
        rows.sort_by(|a, b| a.name.cmp(&b.name));

        let table = Table::new(&rows).with(Style::markdown()).to_string();
        format!(
            "ID:   {}\nSlug: {}\nName: {}\n\n{}",
            self.id, self.slug, self.name, table
        )
    }
}

// --- Public command functions ---

pub async fn list(ctx: &AppContext, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<WorkspaceAttributes> = api.get("workspaces").await?;

    let mut workspaces: Vec<WorkspaceRow> = resp
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

pub async fn show(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<WorkspaceAttributes> = api.get("workspaces").await?;
    let target_lower = target.to_lowercase();

    let workspace = if Uuid::parse_str(target).is_ok() {
        resp.data
            .into_iter()
            .find(|w| w.id.to_lowercase() == target_lower)
            .ok_or_else(|| anyhow::anyhow!("no workspace found with id {}", target))?
    } else {
        resp.data
            .into_iter()
            .find(|w| w.attributes.slug.to_lowercase() == target_lower)
            .ok_or_else(|| anyhow::anyhow!("no workspace found with slug '{}'", target))?
    };

    out.print(&WorkspaceShowOutput {
        id: workspace.id,
        slug: workspace.attributes.slug,
        name: workspace.attributes.name,
        settings: workspace.attributes.settings,
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

    fn workspace_list_response_with_settings(
        workspaces: &[(&str, &str, &str, serde_json::Value)],
    ) -> String {
        let data: Vec<serde_json::Value> = workspaces
            .iter()
            .map(|(id, slug, name, settings)| {
                serde_json::json!({
                    "type": "workspaces",
                    "id": id,
                    "attributes": { "slug": slug, "name": name, "settings": settings }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    // --- list tests ---

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

    // --- show tests ---

    #[tokio::test]
    async fn test_show_by_uuid_plain() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response_with_settings(&[
                (
                    "11111111-1111-1111-1111-111111111111",
                    "cardiology",
                    "Cardiology",
                    serde_json::json!({ "default_for_clinicians": true }),
                ),
                (
                    "22222222-2222-2222-2222-222222222222",
                    "admin",
                    "Administration",
                    serde_json::json!({}),
                ),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(
            &_auth.app_context(&server.url()),
            "11111111-1111-1111-1111-111111111111",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_by_slug_plain() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response_with_settings(&[(
                "11111111-1111-1111-1111-111111111111",
                "cardiology",
                "Cardiology",
                serde_json::json!({ "default_for_clinicians": false }),
            )]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "cardiology", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_json_output() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response_with_settings(&[(
                "11111111-1111-1111-1111-111111111111",
                "cardiology",
                "Cardiology",
                serde_json::json!({ "default_for_clinicians": true }),
            )]))
            .create_async()
            .await;

        let out = Output { json: true };
        let result = show(
            &_auth.app_context(&server.url()),
            "11111111-1111-1111-1111-111111111111",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_target_not_found() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list_response(&[(
                "uuid-1",
                "admin",
                "Administration",
            )]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "no-such-slug", &out).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no workspace found"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_api_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/workspaces")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "cardiology", &out).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }
}
