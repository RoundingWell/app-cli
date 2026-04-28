//! Test fixtures shared across the per-command test modules.

use crate::cli::Stage;
use crate::config::AppContext;
use std::collections::BTreeMap;

/// Writes a fake Bearer token to a temp config dir so that `require_auth` succeeds.
/// The temp directory (and all its contents) is cleaned up on drop.
pub(super) struct TestAuthGuard {
    pub(super) dir: tempfile::TempDir,
}

impl TestAuthGuard {
    pub(super) fn new() -> Self {
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

    pub(super) fn app_context(&self, base_url: &str) -> AppContext {
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

    pub(super) fn app_context_with_defaults(
        &self,
        base_url: &str,
        defaults: BTreeMap<String, String>,
    ) -> AppContext {
        AppContext {
            config_dir: self.dir.path().to_path_buf(),
            profile: "test".to_string(),
            auth_profile: "test".to_string(),
            stage: Stage::Dev,
            auth_stage: Stage::Dev,
            base_url: base_url.to_string(),
            defaults,
        }
    }
}

pub(super) fn clinician_response(id: &str, name: &str, email: &str, enabled: bool) -> String {
    serde_json::json!({
        "data": {
            "type": "clinicians",
            "id": id,
            "attributes": { "name": name, "email": email, "enabled": enabled }
        }
    })
    .to_string()
}

pub(super) fn clinician_list_response(clinicians: &[(&str, &str, &str, bool)]) -> String {
    let data: Vec<serde_json::Value> = clinicians
        .iter()
        .map(|(id, name, email, enabled)| {
            serde_json::json!({
                "type": "clinicians",
                "id": id,
                "attributes": { "name": name, "email": email, "enabled": enabled }
            })
        })
        .collect();
    serde_json::json!({ "data": data }).to_string()
}

pub(super) fn role_list_response(roles: &[(&str, &str)]) -> String {
    let data: Vec<serde_json::Value> = roles
        .iter()
        .map(|(id, name)| {
            serde_json::json!({
                "type": "roles",
                "id": id,
                "attributes": { "name": name }
            })
        })
        .collect();
    serde_json::json!({ "data": data }).to_string()
}

pub(super) fn team_list_response(teams: &[(&str, &str, &str)]) -> String {
    let data: Vec<serde_json::Value> = teams
        .iter()
        .map(|(id, name, abbr)| {
            serde_json::json!({
                "type": "teams",
                "id": id,
                "attributes": { "name": name, "abbr": abbr }
            })
        })
        .collect();
    serde_json::json!({ "data": data }).to_string()
}

pub(super) fn workspace_list_response(workspaces: &[(&str, bool)]) -> String {
    let data: Vec<serde_json::Value> = workspaces
        .iter()
        .map(|(id, default_for_clinicians)| {
            serde_json::json!({
                "type": "workspaces",
                "id": id,
                "attributes": { "settings": { "default_for_clinicians": default_for_clinicians } }
            })
        })
        .collect();
    serde_json::json!({ "data": data }).to_string()
}
