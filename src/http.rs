//! Thin wrapper around `reqwest::Client` for the RoundingWell API.
//!
//! Resolves auth once at construction, attaches it to every request, joins
//! paths against the configured base URL, and centralizes status-check +
//! body-parse error handling.
//!
//! ```ignore
//! let api = ApiClient::new(ctx).await?;
//! let teams: List<TeamAttributes> = api.get("teams").await?;
//! ```

use anyhow::{bail, Context, Result};
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};

use crate::config::AppContext;

/// API client carrying a resolved auth header for the active profile.
pub struct ApiClient<'a> {
    ctx: &'a AppContext,
    client: Client,
    auth_header: String,
}

impl<'a> ApiClient<'a> {
    /// Build a client by resolving auth once. Fails if no credentials are stored.
    pub async fn new(ctx: &'a AppContext) -> Result<Self> {
        let auth_header = crate::commands::auth::require_auth(ctx).await?;
        Ok(Self {
            ctx,
            client: Client::new(),
            auth_header,
        })
    }

    /// Joins `path` to `ctx.base_url`, trimming any extraneous slashes.
    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.ctx.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// `GET {path}` — deserialize the JSON response into `T`.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.send(self.client.get(self.url(path))).await
    }

    /// `GET {path}?query` — `query` is anything `reqwest::RequestBuilder::query` accepts
    /// (commonly `&[(&str, &str)]`).
    pub async fn get_query<T: DeserializeOwned, Q: Serialize + ?Sized>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        self.send(self.client.get(self.url(path)).query(query))
            .await
    }

    /// `PATCH {path}` with `body` serialized as JSON.
    pub async fn patch<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.send(self.client.patch(self.url(path)).json(body))
            .await
    }

    /// `POST {path}` with `body` serialized as JSON.
    pub async fn post<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.send(self.client.post(self.url(path)).json(body)).await
    }

    /// `POST {path}` with `body` serialized as JSON, discarding the response body.
    /// Use for endpoints where success is signalled by status code alone.
    pub async fn post_void<B: Serialize + ?Sized>(&self, path: &str, body: &B) -> Result<()> {
        let req = self.client.post(self.url(path)).json(body);
        let resp = self.auth(req).send().await.context("API request failed")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.context("failed to read response body")?;
            bail!("API returned {}: {}", status, body);
        }
        Ok(())
    }

    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        req.header(reqwest::header::AUTHORIZATION, &self.auth_header)
    }

    async fn send<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T> {
        let resp = self.auth(req).send().await.context("API request failed")?;
        let status = resp.status();
        let body = resp.text().await.context("failed to read response body")?;
        if !status.is_success() {
            bail!("API returned {}: {}", status, body);
        }
        serde_json::from_str(&body).context("failed to parse response body")
    }
}

#[cfg(test)]
impl<'a> ApiClient<'a> {
    /// Test-only constructor that skips the auth-cache lookup.
    pub(crate) fn for_test(ctx: &'a AppContext, auth_header: impl Into<String>) -> Self {
        Self {
            ctx,
            client: Client::new(),
            auth_header: auth_header.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Stage;
    use mockito::{Matcher, Server};
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn ctx(base_url: &str) -> AppContext {
        AppContext {
            config_dir: std::path::PathBuf::from("/tmp"),
            profile: "test".to_string(),
            auth_profile: "test".to_string(),
            stage: Stage::Dev,
            auth_stage: Stage::Dev,
            base_url: base_url.to_string(),
            defaults: BTreeMap::new(),
        }
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Echo {
        ok: bool,
    }

    #[tokio::test]
    async fn test_get_attaches_auth_and_parses_json() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/things")
            .match_header("authorization", "Bearer test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer test-token");
        let resp: Echo = api.get("things").await.unwrap();
        assert_eq!(resp, Echo { ok: true });
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_strips_leading_slash_from_path() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/things")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let _: Echo = api.get("/things").await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_trims_trailing_slash_from_base_url() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/things")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let base = format!("{}/", server.url());
        let c = ctx(&base);
        let api = ApiClient::for_test(&c, "Bearer t");
        let _: Echo = api.get("things").await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_query_passes_query_params() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/things")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("filter[type]".into(), "report".into()),
                Matcher::UrlEncoded("filter[term]".into(), "needle".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let _: Echo = api
            .get_query(
                "things",
                &[("filter[type]", "report"), ("filter[term]", "needle")],
            )
            .await
            .unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_patch_sends_json_body() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PATCH", "/things/1")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(serde_json::json!({"name":"new"})))
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let _: Echo = api
            .patch("things/1", &serde_json::json!({"name": "new"}))
            .await
            .unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_post_sends_json_body() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/things")
            .match_body(Matcher::Json(serde_json::json!({"name":"x"})))
            .with_status(201)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let _: Echo = api
            .post("things", &serde_json::json!({"name": "x"}))
            .await
            .unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_post_void_ignores_response_body() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/things")
            .with_status(204)
            .with_body("")
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        api.post_void("things", &serde_json::json!({"x": 1}))
            .await
            .unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_non_2xx_returns_error_with_body() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/things")
            .with_status(404)
            .with_body("not found")
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let err = api.get::<Echo>("things").await.unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("404"), "expected status in message: {}", msg);
        assert!(
            msg.contains("not found"),
            "expected body in message: {}",
            msg
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_post_void_non_2xx_returns_error_with_body() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/things")
            .with_status(422)
            .with_body("validation failed")
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let err = api
            .post_void("things", &serde_json::json!({}))
            .await
            .unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("422"));
        assert!(msg.contains("validation failed"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_unparseable_json_returns_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/things")
            .with_status(200)
            .with_body("not-json")
            .create_async()
            .await;

        let c = ctx(&server.url());
        let api = ApiClient::for_test(&c, "Bearer t");
        let err = api.get::<Echo>("things").await.unwrap_err();
        assert!(format!("{:#}", err).contains("parse"));
        mock.assert_async().await;
    }
}
