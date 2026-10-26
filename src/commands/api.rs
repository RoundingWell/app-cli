use anyhow::{bail, Context, Result};
use json_dotpath::DotPaths;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};

use crate::cli::ApiArgs;
use crate::config::AppContext;
use crate::output::Output;

/// Default media type for `-f` request bodies.
const JSON_API_MEDIA_TYPE: &str = "application/vnd.api+json";

pub async fn dispatch(args: ApiArgs, ctx: &AppContext, _out: &Output) -> Result<()> {
    run(
        ctx,
        &args.endpoint,
        &args.method,
        &args.headers,
        &args.fields,
        args.raw,
    )
    .await
}

/// Run `rw api <endpoint>` – make an HTTP request to the RoundingWell API.
///
/// # Arguments
/// * `ctx`           – resolved application context (includes base URL, org, stage, auth dir)
/// * `endpoint`      – API path (e.g. `clinicians`, `clinicians/60fda0c4-eca0-434a-80d8-fd4e490aa051`)
/// * `method`        – HTTP verb (e.g. `GET`, `POST`)
/// * `extra_headers` – additional raw header strings (`"Name: value"`)
/// * `fields`        – request body key=value pairs; forces POST when present
/// * `raw`           – if true, print raw JSON; otherwise pretty-print
pub async fn run(
    ctx: &AppContext,
    endpoint: &str,
    method: &str,
    extra_headers: &[String],
    fields: &[String],
    raw: bool,
) -> Result<()> {
    // Strip leading slash from endpoint so we can join cleanly.
    let endpoint = endpoint.trim_start_matches('/');
    let url = format!("{}/{}", ctx.base_url.trim_end_matches('/'), endpoint);

    // Determine the effective HTTP method: if fields are supplied without an
    // explicit override we default to POST.
    let effective_method = if !fields.is_empty() && method.eq_ignore_ascii_case("GET") {
        "POST"
    } else {
        method
    };

    let client = reqwest::Client::new();
    let mut req = client.request(
        effective_method
            .parse()
            .with_context(|| format!("invalid HTTP method: {}", effective_method))?,
        &url,
    );

    // Attach authentication header if credentials are stored.
    req = super::auth::attach_auth(ctx, req).await?;

    // Parse and attach extra headers supplied via -H.
    let mut header_map = HeaderMap::new();
    for h in extra_headers {
        let (name, value) = parse_header(h).with_context(|| format!("invalid header: {:?}", h))?;
        header_map.insert(name, value);
    }
    // Parse -f field=value pairs and attach as a JSON:API body. The API
    // speaks JSON:API, so the default Content-Type is
    // `application/vnd.api+json`; a Content-Type supplied via -H wins
    // (header names are case-insensitive). The body is serialized here
    // because reqwest's `json()` would default to `application/json`.
    if !fields.is_empty() {
        let body =
            serde_json::to_vec(&build_body(fields)?).context("failed to serialize request body")?;
        header_map
            .entry(CONTENT_TYPE)
            .or_insert(HeaderValue::from_static(JSON_API_MEDIA_TYPE));
        req = req.body(body);
    }
    req = req.headers(header_map);

    let resp = req
        .send()
        .await
        .with_context(|| format!("request to {} failed", url))?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    if raw {
        print!("{}", body);
    } else {
        // Pretty-print JSON when possible; fall back to raw output.
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(json) => println!("{}", serde_json::to_string_pretty(&json)?),
            Err(_) => print!("{}", body),
        }
    }

    Ok(())
}

/// Parse a raw `"Name: value"` header string into typed header components.
fn parse_header(s: &str) -> Result<(HeaderName, HeaderValue)> {
    let colon = s
        .find(':')
        .with_context(|| format!("header must be in \"Name: value\" form, got: {:?}", s))?;
    let name_str = s[..colon].trim();
    let value_str = s[colon + 1..].trim();
    let name: HeaderName = name_str
        .parse()
        .with_context(|| format!("invalid header name: {:?}", name_str))?;
    let value: HeaderValue = value_str
        .parse()
        .with_context(|| format!("invalid header value: {:?}", value_str))?;
    Ok((name, value))
}

/// Parse a `"key=value"` field string.
fn parse_field(s: &str) -> Result<(String, String)> {
    let eq = s
        .find('=')
        .with_context(|| format!("field must be in \"key=value\" form, got: {:?}", s))?;
    Ok((s[..eq].to_string(), s[eq + 1..].to_string()))
}

/// Build a nested JSON body from `"key=value"` field strings.
///
/// Keys may use dot-path notation (e.g. `attributes.name`) to produce nested
/// objects. Multiple fields that share a common prefix are merged. Returns an
/// error when a dot-path key conflicts with an already-set leaf value.
fn build_body(fields: &[String]) -> Result<serde_json::Value> {
    let mut body = serde_json::Value::Object(serde_json::Map::new());
    for f in fields {
        let (k, v) = parse_field(f).with_context(|| format!("invalid field: {:?}", f))?;
        body.dot_set(&k, serde_json::Value::String(v))
            .map_err(|_| {
                anyhow::anyhow!(
                    "Unable to set field {} because it conflicts with another field",
                    k
                )
            })?;
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    // --- run / request tests ---

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
            AppContext {
                config_dir: self.dir.path().to_path_buf(),
                profile: "test".to_string(),
                auth_profile: "test".to_string(),
                stage: Stage::Dev,
                auth_stage: Stage::Dev,
                base_url: base_url.to_string(),
                defaults: std::collections::BTreeMap::new(),
            }
        }
    }

    /// Send `rw api -X PUT clinicians/1 -f attributes.name=Jane` with the
    /// given -H headers and assert the server saw `expected_content_type`
    /// alongside the JSON body.
    async fn assert_put_content_type(headers: &[&str], expected_content_type: &str) {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/clinicians/1")
            .match_header("content-type", expected_content_type)
            .match_body(Matcher::Json(
                serde_json::json!({"attributes": {"name": "Jane"}}),
            ))
            .with_status(200)
            .with_body("{}")
            .expect(1)
            .create_async()
            .await;

        let guard = TestAuthGuard::new();
        let ctx = guard.app_context(&server.url());
        let headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        run(
            &ctx,
            "clinicians/1",
            "PUT",
            &headers,
            &["attributes.name=Jane".to_string()],
            true,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_run_fields_default_content_type_json_api() {
        assert_put_content_type(&[], "application/vnd.api+json").await;
    }

    #[tokio::test]
    async fn test_run_fields_keep_caller_content_type() {
        assert_put_content_type(&["Content-Type: application/json"], "application/json").await;
    }

    #[tokio::test]
    async fn test_run_fields_keep_caller_content_type_any_case() {
        assert_put_content_type(&["content-TYPE: application/json"], "application/json").await;
    }

    #[tokio::test]
    async fn test_run_fields_keep_other_headers_with_default_content_type() {
        assert_put_content_type(&["Accept: application/json"], "application/vnd.api+json").await;
    }

    // --- build_body / dot-path tests ---

    #[test]
    fn test_build_body_dot_path_single() {
        let body = build_body(&["attributes.name=John".to_string()]).unwrap();
        assert_eq!(body, serde_json::json!({"attributes": {"name": "John"}}));
    }

    #[test]
    fn test_build_body_dot_path_shared_prefix() {
        let body = build_body(&[
            "attributes.first=John".to_string(),
            "attributes.last=Doe".to_string(),
        ])
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"attributes": {"first": "John", "last": "Doe"}})
        );
    }

    #[test]
    fn test_build_body_dot_path_deep() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let body = build_body(&[format!("relationships.team.data.id={id}")]).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"relationships": {"team": {"data": {"id": id}}}})
        );
    }

    #[test]
    fn test_build_body_mixed_flat_and_dot_path() {
        let body = build_body(&[
            "type=clinicians".to_string(),
            "attributes.name=Jane".to_string(),
        ])
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"type": "clinicians", "attributes": {"name": "Jane"}})
        );
    }

    #[test]
    fn test_build_body_dot_in_value_not_split() {
        let body = build_body(&["attributes.email=user@example.com".to_string()]).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"attributes": {"email": "user@example.com"}})
        );
    }

    #[test]
    fn test_field_integer_segment_creates_array() {
        let body = build_body(&["items.0.id=abc".to_string()]).unwrap();
        assert_eq!(body, serde_json::json!({"items": [{"id": "abc"}]}));
    }

    #[test]
    fn test_field_multiple_integer_segments() {
        let body =
            build_body(&["items.0.id=abc".to_string(), "items.1.id=def".to_string()]).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"items": [{"id": "abc"}, {"id": "def"}]})
        );
    }

    #[test]
    fn test_field_integer_segment_nested() {
        let uuid = "uuid-123";
        let body = build_body(&[
            "relationships.workspaces.data.0.type=workspaces".to_string(),
            format!("relationships.workspaces.data.0.id={uuid}"),
        ])
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"relationships": {"workspaces": {"data": [{"type": "workspaces", "id": uuid}]}}})
        );
    }

    #[test]
    fn test_build_body_key_conflict_error() {
        let result = build_body(&["foo=bar".to_string(), "foo.baz=qux".to_string()]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert_eq!(
            msg,
            "Unable to set field foo.baz because it conflicts with another field"
        );
    }

    #[test]
    fn test_build_body_key_conflict_error_multilevel() {
        let result = build_body(&["a.b=x".to_string(), "a.b.c=y".to_string()]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert_eq!(
            msg,
            "Unable to set field a.b.c because it conflicts with another field"
        );
    }

    // --- existing parse_header / parse_field tests ---

    #[test]
    fn test_parse_header_valid() {
        let (name, value) = parse_header("Accept: application/json").unwrap();
        assert_eq!(name, "accept");
        assert_eq!(value, "application/json");
    }

    #[test]
    fn test_parse_header_invalid() {
        assert!(parse_header("no-colon").is_err());
    }

    #[test]
    fn test_parse_field_valid() {
        let (k, v) = parse_field("key=value").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "value");
    }

    #[test]
    fn test_parse_field_value_with_equals() {
        let (k, v) = parse_field("key=val=ue").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "val=ue");
    }

    #[test]
    fn test_parse_field_invalid() {
        assert!(parse_field("no-equals").is_err());
    }
}
