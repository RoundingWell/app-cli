//! Stage enum + methods for resolving its API URL and WorkOS config.

use clap::ValueEnum;

/// Stage value for the --stage global option.
#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Prod,
    Sandbox,
    Qa,
    Dev,
    Local,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Prod => write!(f, "prod"),
            Stage::Sandbox => write!(f, "sandbox"),
            Stage::Qa => write!(f, "qa"),
            Stage::Dev => write!(f, "dev"),
            Stage::Local => write!(f, "local"),
        }
    }
}

impl Stage {
    /// Resolves the API root URL for this stage and `organization`.
    pub fn api_url(&self, organization: &str) -> String {
        match self {
            // Local access is direct, no `/api` suffix.
            Stage::Local => "http://localhost:8080".to_string(),
            // Live access mounts the API with an `/api` suffix.
            Stage::Dev => format!("https://{}.roundingwell.dev/api", organization),
            Stage::Sandbox => format!("https://{}-sandbox.roundingwell.com/api", organization),
            _ => format!("https://{}.roundingwell.com/api", organization),
        }
    }

    /// Returns the WorkOS AuthKit configuration for this stage. Sandbox and
    /// Prod share the production tenant; everything else uses staging.
    pub fn workos_config(&self) -> &'static WorkOsConfig {
        match self {
            Stage::Prod | Stage::Sandbox => &PRODUCTION,
            _ => &DEV,
        }
    }
}

/// WorkOS AuthKit endpoints + client id for one of the two tenants.
pub struct WorkOsConfig {
    pub client_id: &'static str,
    pub device_auth_url: &'static str,
    pub token_url: &'static str,
}

const PRODUCTION: WorkOsConfig = WorkOsConfig {
    client_id: "client_01KMREY0MMNCB4B9AK4X9C0TBG",
    device_auth_url: "https://authkit.roundingwell.com/oauth2/device_authorization",
    token_url: "https://authkit.roundingwell.com/oauth2/token",
};

const DEV: WorkOsConfig = WorkOsConfig {
    client_id: "client_01KMRQT9V7YE17BYA5NDSPK572",
    device_auth_url: "https://expansive-market-28-staging.authkit.app/oauth2/device_authorization",
    token_url: "https://expansive-market-28-staging.authkit.app/oauth2/token",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prod_api_url() {
        assert_eq!(
            Stage::Prod.api_url("demonstration"),
            "https://demonstration.roundingwell.com/api"
        );
    }

    #[test]
    fn test_qa_api_url() {
        assert_eq!(Stage::Qa.api_url("qa2"), "https://qa2.roundingwell.com/api");
    }

    #[test]
    fn test_dev_api_url() {
        assert_eq!(
            Stage::Dev.api_url("jane"),
            "https://jane.roundingwell.dev/api"
        );
    }

    #[test]
    fn test_sandbox_api_url() {
        assert_eq!(
            Stage::Sandbox.api_url("mercy"),
            "https://mercy-sandbox.roundingwell.com/api"
        );
    }

    #[test]
    fn test_local_api_url() {
        assert_eq!(Stage::Local.api_url("rw"), "http://localhost:8080");
    }

    #[test]
    fn test_workos_config_prod_uses_production_tenant() {
        let cfg = Stage::Prod.workos_config();
        assert!(cfg.token_url.contains("authkit.roundingwell.com"));
    }

    #[test]
    fn test_workos_config_sandbox_uses_production_tenant() {
        let cfg = Stage::Sandbox.workos_config();
        assert!(cfg.token_url.contains("authkit.roundingwell.com"));
    }

    #[test]
    fn test_workos_config_dev_uses_staging_tenant() {
        let cfg = Stage::Dev.workos_config();
        assert!(cfg.token_url.contains("staging"));
    }
}
