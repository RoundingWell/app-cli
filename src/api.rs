//! Thin facade over `crate::domain::stage::Stage::api_url`. Retained as a
//! free function so existing call sites that imported `resolve_api` continue
//! to work.

use crate::cli::Stage;

pub fn resolve_api(organization: &str, stage: &Stage) -> String {
    stage.api_url(organization)
}
