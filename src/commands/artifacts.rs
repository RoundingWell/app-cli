use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppContext;
use crate::output::{CommandOutput, Output};

// --- JSON:API deserialization types ---

#[derive(Debug, Deserialize)]
pub(crate) struct ArtifactAttributes {
    pub(crate) artifact: String,
    pub(crate) identifier: String,
    pub(crate) values: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArtifactResource {
    pub(crate) attributes: ArtifactAttributes,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArtifactListResponse {
    pub(crate) data: Vec<ArtifactResource>,
}

// --- Output types ---

#[derive(Debug, tabled::Tabled, Serialize)]
pub struct ArtifactRow {
    pub artifact: String,
    pub identifier: String,
    pub values: String,
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
        Table::new(&self.artifacts)
            .with(Style::markdown())
            .to_string()
    }
}

// --- Public command functions ---

pub async fn list(
    ctx: &AppContext,
    r#type: &str,
    path: &str,
    term: &str,
    out: &Output,
) -> Result<()> {
    let auth_header = super::auth::require_auth(ctx).await?;
    let client = Client::new();

    let url = format!("{}/artifacts", ctx.base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, &auth_header)
        .query(&[
            ("filter[type]", r#type),
            ("filter[path]", path),
            ("filter[term]", term),
        ])
        .send()
        .await
        .context("GET /artifacts failed")?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        anyhow::bail!("API returned {}: {}", status, body);
    }

    let list: ArtifactListResponse =
        serde_json::from_str(&body).context("failed to parse artifacts response")?;

    let artifacts: Vec<ArtifactRow> = list
        .data
        .into_iter()
        .map(|a| ArtifactRow {
            artifact: a.attributes.artifact,
            identifier: a.attributes.identifier,
            values: a.attributes.values.join(", "),
        })
        .collect();

    out.print(&ArtifactListOutput { artifacts });
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

    fn artifact_list_response(artifacts: &[(&str, &str, &[&str])]) -> String {
        let data: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|(artifact, identifier, values)| {
                serde_json::json!({
                    "type": "artifacts",
                    "id": format!("artifact-{}", identifier),
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
    async fn test_list_artifacts() {
        let auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("filter[type]".to_string(), "icd".to_string()),
                mockito::Matcher::UrlEncoded("filter[path]".to_string(), "some/path".to_string()),
                mockito::Matcher::UrlEncoded("filter[term]".to_string(), "diabetes".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[
                ("ICD-10", "E11", &["Type 2 diabetes mellitus"]),
                ("ICD-10", "E10", &["Type 1 diabetes mellitus"]),
            ]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(
            &auth.app_context(&server.url()),
            "icd",
            "some/path",
            "diabetes",
            &out,
        )
        .await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_artifacts_empty() {
        let auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("filter[type]".to_string(), "icd".to_string()),
                mockito::Matcher::UrlEncoded("filter[path]".to_string(), "p".to_string()),
                mockito::Matcher::UrlEncoded("filter[term]".to_string(), "xyz".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&auth.app_context(&server.url()), "icd", "p", "xyz", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_artifacts_multiple_values() {
        let auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("filter[type]".to_string(), "icd".to_string()),
                mockito::Matcher::UrlEncoded("filter[path]".to_string(), "p".to_string()),
                mockito::Matcher::UrlEncoded("filter[term]".to_string(), "t".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[(
                "ICD-10",
                "E11.9",
                &["Type 2 diabetes", "unspecified"],
            )]))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&auth.app_context(&server.url()), "icd", "p", "t", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_artifacts_api_error() {
        let auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::AnyOf(vec![mockito::Matcher::Any]))
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let out = Output { json: false };
        let result = list(&auth.app_context(&server.url()), "icd", "p", "t", &out).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_artifacts_json_output() {
        let auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/artifacts")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("filter[type]".to_string(), "icd".to_string()),
                mockito::Matcher::UrlEncoded("filter[path]".to_string(), "p".to_string()),
                mockito::Matcher::UrlEncoded("filter[term]".to_string(), "t".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(artifact_list_response(&[(
                "ICD-10",
                "E11",
                &["Type 2 diabetes mellitus"],
            )]))
            .create_async()
            .await;

        let out = Output { json: true };
        let result = list(&auth.app_context(&server.url()), "icd", "p", "t", &out).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }
}
