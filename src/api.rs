use crate::cli::Stage;

/// Resolves the API root URL for the given organization and stage.
pub fn resolve_api(organization: &str, stage: &Stage) -> String {
    match stage {
        // Local access is direct, no `/api` suffix.
        Stage::Local => "http://localhost:8080".to_string(),
        // Live access mounts the API with an `/api` suffix.
        Stage::Dev => format!("https://{}.roundingwell.dev/api", organization),
        Stage::Sandbox => format!("https://{}-sandbox.roundingwell.com/api", organization),
        _ => format!("https://{}.roundingwell.com/api", organization),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prod_api() {
        assert_eq!(
            resolve_api("demonstration", &Stage::Prod),
            "https://demonstration.roundingwell.com/api"
        );
    }

    #[test]
    fn test_qa_api() {
        assert_eq!(
            resolve_api("qa2", &Stage::Qa),
            "https://qa2.roundingwell.com/api"
        );
    }

    #[test]
    fn test_dev_api() {
        assert_eq!(
            resolve_api("jane", &Stage::Dev),
            "https://jane.roundingwell.dev/api"
        );
    }

    #[test]
    fn test_sandbox_api() {
        assert_eq!(
            resolve_api("mercy", &Stage::Sandbox),
            "https://mercy-sandbox.roundingwell.com/api"
        );
    }

    #[test]
    fn test_local_api() {
        assert_eq!(resolve_api("rw", &Stage::Local), "http://localhost:8080");
    }
}
