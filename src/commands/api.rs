use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::config::{AuthEntry, Config};

/// Run `rw api <endpoint>` – make an HTTP request to the RoundingWell API.
///
/// # Arguments
/// * `config`        – loaded configuration (for authentication lookup)
/// * `base_url`      – resolved API URI (e.g. `https://demonstration.roundingwell.com/api`)
/// * `profile`       – profile name used to look up auth credentials
/// * `endpoint`      – API path (e.g. `clinicians`, `clinicians/60fda0c4-eca0-434a-80d8-fd4e490aa051`)
/// * `method`        – HTTP verb (e.g. `GET`, `POST`)
/// * `extra_headers` – additional raw header strings (`"Name: value"`)
/// * `fields`        – request body key=value pairs; forces POST when present
/// * `jq`            – optional jq filter expression to apply to the response
/// * `raw`           – if true, print raw JSON; otherwise pretty-print
pub async fn run(
    config: &Config,
    base_url: &str,
    profile: &str,
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
    if let Some(auth) = config.authentication.get(profile) {
        match auth {
            AuthEntry::Bearer { bearer } => {
                req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", bearer));
            }
            AuthEntry::Basic { basic } => {
                req = req.basic_auth(&basic.username, Some(&basic.password));
            }
        }
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

/// Pipe `json` through `jq` with the given filter and return the output.
fn apply_jq(filter: &str, json: &str) -> Result<String> {
    let mut child = Command::new("jq")
        .arg(filter)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run jq; is it installed?")?;

    child
        .stdin
        .take()
        .unwrap()
        .write_all(json.as_bytes())
        .context("failed to write to jq stdin")?;

    let output = child.wait_with_output().context("failed to wait for jq")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("jq: {}", stderr.trim());
    }

    String::from_utf8(output.stdout).context("jq output was not valid UTF-8")
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
}
