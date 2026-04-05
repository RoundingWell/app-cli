use anyhow::{bail, Context, Result};
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{data, unwrap_valr, Compiler, Ctx, Vars};
use jaq_json::{read, Val};
use json_dotpath::DotPaths;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::config::AppContext;

/// Run `rw api <endpoint>` – make an HTTP request to the RoundingWell API.
///
/// # Arguments
/// * `ctx`           – resolved application context (includes base URL, org, stage, auth dir)
/// * `endpoint`      – API path (e.g. `clinicians`, `clinicians/60fda0c4-eca0-434a-80d8-fd4e490aa051`)
/// * `method`        – HTTP verb (e.g. `GET`, `POST`)
/// * `extra_headers` – additional raw header strings (`"Name: value"`)
/// * `fields`        – request body key=value pairs; forces POST when present
/// * `jq`            – optional jq filter expression to apply to the response
/// * `raw`           – if true, print raw JSON; otherwise pretty-print
pub async fn run(
    ctx: &AppContext,
    endpoint: &str,
    method: &str,
    extra_headers: &[String],
    fields: &[String],
    jq: Option<&str>,
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
    req = req.headers(header_map);

    // Parse -f field=value pairs and attach as JSON body.
    if !fields.is_empty() {
        let body = build_body(fields)?;
        req = req.json(&body);
    }

    let resp = req
        .send()
        .await
        .with_context(|| format!("request to {} failed", url))?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if !status.is_success() {
        bail!("API returned {}: {}", status, body);
    }

    if let Some(filter) = jq {
        print!("{}", apply_jq(filter, &body)?);
    } else if raw {
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

/// Apply a jq filter expression to `json` using the `jaq-core` library.
fn apply_jq(filter_str: &str, json: &str) -> Result<String> {
    let input = read::parse_single(json.as_bytes())
        .map_err(|e| anyhow::anyhow!("jq: invalid JSON input: {e}"))?;

    let program = File {
        code: filter_str,
        path: (),
    };

    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());

    let loader = Loader::new(defs);
    let arena = Arena::default();

    let modules = loader
        .load(&arena, program)
        .map_err(|errs| anyhow::anyhow!("jq: {}", format_load_errors(errs)))?;

    let filter = Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errs| anyhow::anyhow!("jq: {errs:?}"))?;

    let ctx = Ctx::<data::JustLut<Val>>::new(&filter.lut, Vars::new([]));
    let out = filter.id.run((ctx, input)).map(unwrap_valr);

    let mut output = String::new();
    for result in out {
        let val = result.map_err(|e| anyhow::anyhow!("jq: {e}"))?;
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&val.to_string());
    }
    if !output.is_empty() {
        output.push('\n');
    }

    Ok(output)
}

fn format_load_errors<E: std::fmt::Debug>(errs: E) -> String {
    format!("{errs:?}")
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
            format!("relationships.workspaces.data.0.type=workspaces"),
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

    #[test]
    fn test_apply_jq_identity() {
        let result = apply_jq(".", r#"{"a":1}"#).unwrap();
        assert_eq!(result.trim(), r#"{"a":1}"#);
    }

    #[test]
    fn test_apply_jq_field_access() {
        let result = apply_jq(".name", r#"{"name":"Alice"}"#).unwrap();
        assert_eq!(result.trim(), r#""Alice""#);
    }

    #[test]
    fn test_apply_jq_array_iterator() {
        let result = apply_jq(".[]", r#"[1,2,3]"#).unwrap();
        assert_eq!(result.trim(), "1\n2\n3");
    }

    #[test]
    fn test_apply_jq_invalid_filter() {
        assert!(apply_jq("invalid!!!", "{}").is_err());
    }

    #[test]
    fn test_apply_jq_invalid_json() {
        assert!(apply_jq(".", "not-json").is_err());
    }
}
