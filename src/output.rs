use serde::Serialize;

/// Implemented by all command output types.
/// `plain()` returns the human-readable representation; `Serialize` provides the JSON one.
pub trait CommandOutput: Serialize {
    fn plain(&self) -> String;
}

/// Carries the `--json` flag and routes output accordingly.
pub struct Output {
    pub json: bool,
}

impl Output {
    pub fn print<T: CommandOutput>(&self, data: &T) {
        if self.json {
            println!("{}", serde_json::json!({ "data": data }));
        } else {
            println!("{}", data.plain());
        }
    }

    /// Print an informational progress message. Silently discarded in JSON mode.
    pub fn info(&self, msg: &str) {
        if !self.json {
            println!("{}", msg);
        }
    }

    /// Print a warning to stderr. Always emitted, even in JSON mode.
    pub fn warn(&self, msg: &str) {
        eprintln!("{}", msg);
    }

    pub fn error(&self, err: &anyhow::Error) {
        if self.json {
            eprintln!("{}", serde_json::json!({ "error": format!("{:#}", err) }));
        } else {
            eprintln!("Error: {:#}", err);
        }
    }
}
