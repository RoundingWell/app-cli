use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::List;
use crate::output::{CommandOutput, Output};

// --- JSON:API attributes ---

#[derive(Debug, Deserialize)]
pub(crate) struct RoleAttributes {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) permissions: Vec<String>,
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

// --- Show output type ---

#[derive(Debug, Serialize)]
pub struct RoleShowOutput {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: String,
    pub permissions: Vec<String>,
}

impl CommandOutput for RoleShowOutput {
    fn plain(&self) -> String {
        let mut lines = vec![
            format!("ID:          {}", self.id),
            format!("Name:        {}", self.name),
            format!("Label:       {}", self.label),
            format!("Description: {}", self.description),
            "Permissions:".to_string(),
        ];
        if self.permissions.is_empty() {
            lines.push("  (none)".to_string());
        } else {
            for p in &self.permissions {
                lines.push(format!("  - {}", p));
            }
        }
        lines.join("\n")
    }
}

// --- Public command functions ---

pub async fn list(ctx: &AppContext, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<RoleAttributes> = api.get("roles").await?;

    let mut roles: Vec<RoleRow> = resp
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

/// Resolves a role by UUID or name; returns `(id, name)`. Used by clinician
/// commands.
pub(crate) async fn resolve_role(
    api: &ApiClient<'_>,
    role_target: &str,
) -> Result<(String, String)> {
    let resp: List<RoleAttributes> = api.get("roles").await?;
    let target_lower = role_target.to_lowercase();

    if Uuid::parse_str(role_target).is_ok() {
        resp.data
            .into_iter()
            .find(|r| r.id.to_lowercase() == target_lower)
            .map(|r| (r.id, r.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no role found with id {}", role_target))
    } else {
        resp.data
            .into_iter()
            .find(|r| r.attributes.name.to_lowercase() == target_lower)
            .map(|r| (r.id, r.attributes.name))
            .ok_or_else(|| anyhow::anyhow!("no role found with name '{}'", role_target))
    }
}

pub async fn show(ctx: &AppContext, target: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<RoleAttributes> = api.get("roles").await?;
    let target_lower = target.to_lowercase();

    let role = if Uuid::parse_str(target).is_ok() {
        resp.data
            .into_iter()
            .find(|r| r.id.to_lowercase() == target_lower)
            .ok_or_else(|| anyhow::anyhow!("no role found with id {}", target))?
    } else {
        resp.data
            .into_iter()
            .find(|r| r.attributes.name.to_lowercase() == target_lower)
            .ok_or_else(|| anyhow::anyhow!("no role found with name '{}'", target))?
    };

    let mut permissions = role.attributes.permissions;
    permissions.sort();
    out.print(&RoleShowOutput {
        id: role.id,
        name: role.attributes.name,
        label: role.attributes.label,
        description: role.attributes.description,
        permissions,
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

    fn role_list_response_full(roles: &[(&str, &str, &str, &str, &[&str])]) -> String {
        let data: Vec<serde_json::Value> = roles
            .iter()
            .map(|(id, name, label, description, permissions)| {
                serde_json::json!({
                    "type": "roles",
                    "id": id,
                    "attributes": {
                        "name": name,
                        "label": label,
                        "description": description,
                        "permissions": permissions
                    }
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

    #[tokio::test]
    async fn test_show_by_uuid() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response_full(&[(
                "aaaaaaaa-0000-0000-0000-000000000001",
                "admin",
                "Administrator",
                "Full access",
                &["write", "read", "delete"],
            )]))
            .create_async()
            .await;

        let out = Output { json: true };
        let output = show(
            &_auth.app_context(&server.url()),
            "aaaaaaaa-0000-0000-0000-000000000001",
            &out,
        )
        .await;
        assert!(output.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_permissions_sorted() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response_full(&[(
                "aaaaaaaa-0000-0000-0000-000000000001",
                "admin",
                "Administrator",
                "Full access",
                &["write", "read", "delete"],
            )]))
            .create_async()
            .await;

        let auth_header = "Bearer test-token";
        let client = reqwest::Client::new();
        let url = format!("{}/roles", server.url());
        let resp = client
            .get(&url)
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .send()
            .await
            .unwrap();
        let body = resp.text().await.unwrap();
        let list: List<RoleAttributes> = serde_json::from_str(&body).unwrap();
        let role = list.data.into_iter().next().unwrap();
        let mut permissions = role.attributes.permissions;
        permissions.sort();
        assert_eq!(permissions, vec!["delete", "read", "write"]);
    }

    #[tokio::test]
    async fn test_show_by_name() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response_full(&[(
                "aaaaaaaa-0000-0000-0000-000000000002",
                "nurse",
                "Nurse",
                "Clinical access",
                &["read"],
            )]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "nurse", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_target_not_found() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(role_list_response_full(&[(
                "aaaaaaaa-0000-0000-0000-000000000001",
                "admin",
                "Administrator",
                "",
                &[],
            )]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "nonexistent", &out).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no role found"), "unexpected error: {err}");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_show_api_error() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/roles")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let out = Output { json: false };
        let result = show(&_auth.app_context(&server.url()), "admin", &out).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }
}
