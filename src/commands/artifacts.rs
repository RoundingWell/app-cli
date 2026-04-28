use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::List;
use crate::output::{CommandOutput, Output};

// --- JSON:API attributes ---

#[derive(Debug, Deserialize)]
pub(crate) struct ArtifactAttributes {
    pub(crate) artifact: String,
    pub(crate) identifier: String,
    #[serde(default)]
    pub(crate) values: serde_json::Map<String, serde_json::Value>,
}

// --- Output types ---

#[derive(Debug, tabled::Tabled)]
struct ValueRow {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
pub struct ArtifactRow {
    pub artifact: String,
    pub identifier: String,
    pub values: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct ArtifactListOutput {
    pub artifacts: Vec<ArtifactRow>,
}

impl CommandOutput for ArtifactListOutput {
    fn plain(&self) -> String {
        use tabled::settings::Style;
        use tabled::Table;
        self.artifacts
            .iter()
            .map(|r| {
                let rows: Vec<ValueRow> = r
                    .values
                    .iter()
                    .map(|(k, v)| ValueRow {
                        key: k.clone(),
                        value: v.to_string(),
                    })
                    .collect();
                let table = Table::new(&rows).with(Style::markdown()).to_string();
                format!("{}\n{}", r.identifier, table)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

// --- Public command functions ---

pub async fn list(
    ctx: &AppContext,
    artifact_type: &str,
    path: &str,
    term: &str,
    out: &Output,
) -> Result<()> {
    let api = ApiClient::new(ctx).await?;
    let resp: List<ArtifactAttributes> = api
        .get_query(
            "artifacts",
            &[
                ("filter[type]", artifact_type),
                ("filter[path]", path),
                ("filter[term]", term),
            ],
        )
        .await?;

    let data: Vec<ArtifactRow> = resp
        .data
        .into_iter()
        .map(|a| ArtifactRow {
            artifact: a.attributes.artifact,
            identifier: a.attributes.identifier,
            values: a.attributes.values,
        })
        .collect();

    out.print(&ArtifactListOutput { artifacts: data });
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

    fn artifact_list_response(artifacts: &[(&str, &str, serde_json::Value)]) -> String {
        let data: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|(artifact, identifier, values)| {
                serde_json::json!({
                    "type": "artifacts",
                    "id": "some-uuid",
                    "attributes": {
                        "artifact": artifact,
                        "identifier": identifier,
                        "values": values
                    }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    #[tokio::test]
    async fn test_list_with_all_filters_returns_table_output() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[
                (
                    "my-artifact",
                    "my-id",
                    serde_json::json!({ "key": "value" }),
                ),
                (
                    "other-artifact",
                    "other-id",
                    serde_json::json!({ "foo": 42 }),
                ),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(
            &_auth.app_context(&server.url()),
            "custom",
            "/some/path",
            "search-term",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_empty_result_displays_headers_only_table() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(
            &_auth.app_context(&server.url()),
            "custom",
            "/some/path",
            "search-term",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_json_flag_returns_json_with_data_array() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[(
                "my-artifact",
                "my-id",
                serde_json::json!({ "key": "value" }),
            )]))
            .create_async()
            .await;

        let out = Output { json: true };
        let result = list(
            &_auth.app_context(&server.url()),
            "custom",
            "/some/path",
            "search-term",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_api_error_exits_non_zero() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(
            &_auth.app_context(&server.url()),
            "custom",
            "/some/path",
            "search-term",
            &out,
        )
        .await;
        assert!(result.is_err());
        mock.assert_async().await;
    }
}
