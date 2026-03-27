use crate::cli::Stage;

/// Resolves the base domain URL for the given organization and stage.
pub fn resolve_domain(organization: &str, stage: &Stage) -> String {
    match stage {
        Stage::Prod | Stage::Qa => format!("https://{}.roundingwell.com", organization),
        Stage::Dev => format!("https://{}.roundingwell.dev", organization),
        Stage::Sandbox => format!("https://{}-sandbox.roundingwell.com", organization),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prod_domain() {
        assert_eq!(
            resolve_domain("demonstration", &Stage::Prod),
            "https://demonstration.roundingwell.com"
        );
    }

    #[test]
    fn test_qa_domain() {
        assert_eq!(
            resolve_domain("myorg", &Stage::Qa),
            "https://myorg.roundingwell.com"
        );
    }

    #[test]
    fn test_dev_domain() {
        assert_eq!(
            resolve_domain("myorg", &Stage::Dev),
            "https://myorg.roundingwell.dev"
        );
    }

    #[test]
    fn test_sandbox_domain() {
        assert_eq!(
            resolve_domain("myorg", &Stage::Sandbox),
            "https://myorg-sandbox.roundingwell.com"
        );
    }
}
