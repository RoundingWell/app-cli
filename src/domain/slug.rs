//! Slug validation: lowercase ASCII identifier with optional internal hyphens.

use std::sync::LazyLock;

use regex::Regex;

static SLUG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9-]*[a-z0-9]$").unwrap());

/// Validate that `s` is a valid slug matching `^[a-z][a-z0-9-]*[a-z0-9]$`.
/// Returns `Ok(s.to_string())` on success.
pub fn validate_slug(s: &str) -> Result<String, String> {
    if s.len() < 2 {
        return Err(format!(
            "'{}' is too short; slugs must be at least 2 characters",
            s
        ));
    }
    if SLUG_RE.is_match(s) {
        Ok(s.to_string())
    } else {
        Err(format!(
            "'{}' is not a valid slug; must match ^[a-z][a-z0-9-]*[a-z0-9]$",
            s
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slugs() {
        assert!(validate_slug("demonstration").is_ok());
        assert!(validate_slug("mercy-clinic").is_ok());
        assert!(validate_slug("qa2").is_ok());
        assert!(validate_slug("qa").is_ok());
    }

    #[test]
    fn test_invalid_slugs() {
        assert!(validate_slug("a").is_err()); // too short
        assert!(validate_slug("Mercy-Clinic").is_err()); // uppercase
        assert!(validate_slug("Mercy Clinic").is_err()); // space not allowed
        assert!(validate_slug("-clinic").is_err()); // starts with hyphen
        assert!(validate_slug("mercy-").is_err()); // ends with hyphen
        assert!(validate_slug("mercy_clinic").is_err()); // underscore not allowed
    }
}
