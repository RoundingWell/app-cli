use anyhow::Result;
use serde::Serialize;

use crate::output::{CommandOutput, Output};
use crate::version_check::do_update;

#[derive(Debug, Serialize)]
pub struct UpdateOutput {
    pub version: String,
    pub updated: bool,
}

impl CommandOutput for UpdateOutput {
    fn plain(&self) -> String {
        if self.updated {
            format!("Updated rw to {}.", self.version)
        } else {
            format!("Already up to date (rw {}).", self.version)
        }
    }
}

pub async fn run(out: &Output) -> Result<()> {
    out.info("Checking for updates...");
    let (version, updated) = match tokio::task::spawn_blocking(do_update).await {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => anyhow::bail!("Update failed: {:#}", e),
        Err(_) => anyhow::bail!("Update task panicked"),
    };
    out.print(&UpdateOutput { version, updated });
    Ok(())
}

pub async fn dispatch(out: &Output) -> Result<()> {
    run(out).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_output_plain_updated() {
        let output = UpdateOutput {
            version: "1.2.3".to_string(),
            updated: true,
        };
        assert_eq!(output.plain(), "Updated rw to 1.2.3.");
    }

    #[test]
    fn test_update_output_plain_already_up_to_date() {
        let output = UpdateOutput {
            version: "1.2.3".to_string(),
            updated: false,
        };
        assert_eq!(output.plain(), "Already up to date (rw 1.2.3).");
    }

    #[test]
    fn test_update_output_json() {
        let output = UpdateOutput {
            version: "1.2.3".to_string(),
            updated: true,
        };
        let value = serde_json::to_value(&output).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"version": "1.2.3", "updated": true})
        );
    }
}
