use anyhow::{bail, Context, Result};
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{data, unwrap_valr, Compiler, Ctx, Vars};
use jaq_json::{read, Val};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

use crate::cli::Stage;

/// Run `rw api <endpoint>` – make an HTTP request to the RoundingWell API.
///
/// # Arguments
/// * `base_url`      – resolved API URI (e.g. `https://demonstration.roundingwell.com/api`)
/// * `org`           – organization slug, used to resolve auth credentials
/// * `stage`         – deployment stage, used to resolve auth credentials
/// * `endpoint`      – API path (e.g. `clinicians`, `clinicians/60fda0c4-eca0-434a-80d8-fd4e490aa051`)
/// * `method`        – HTTP verb (e.g. `GET`, `POST`)
/// * `extra_headers` – additional raw header strings (`"Name: value"`)
/// * `fields`        – request body key=value pairs; forces POST when present
/// * `jq`            – optional jq filter expression to apply to the response
/// * `raw`           – if true, print raw JSON; otherwise pretty-print
pub async fn run(
    base_url: &str,
    org: &str,
    stage: &Stage,
    endpoint: &str,
    method: &str,
    extra_headers: &[String],
    fields: &[String],
    jq: Option<&str>,
    raw: bool,
) -> Result<()> {
    // Strip leading slash from endpoint so we can join cleanly.
    let endpoint = endpoint.trim_start_matches('/');
    let url = format!("{}/{}", base_url.trim_end_matches('/'), endpoint);

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
    match super::auth::resolve_auth(org, stage).await? {
        Some(super::auth::ResolvedAuth::Bearer(token)) => {
            req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token));
        }
        Some(super::auth::ResolvedAuth::Basic { username, password }) => {
            req = req.basic_auth(&username, Some(&password));
        }
        None => {}
    }

    // Parse and attach extra headers supplied via -H.
    let mut header_map = HeaderMap::new();
    for h in extra_headers {
        let (name, value) = parse_header(h).with_context(|| format!("invalid header: {:?}", h))?;
        header_map.insert(name, value);
    }
    req = req.headers(header_map);

    // Parse -f field=value pairs and attach as JSON body.
    if !fields.is_empty() {
        let mut body: HashMap<String, String> = HashMap::new();
        for f in fields {
            let (k, v) = parse_field(f).with_context(|| format!("invalid field: {:?}", f))?;
            body.insert(k, v);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

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
