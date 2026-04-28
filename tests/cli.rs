//! Integration tests that spawn the `rw` binary end-to-end.
//!
//! These tests cover the CLI shell: argument parsing, dispatch, error
//! reporting, and exit codes — paths that unit tests inside the library can't
//! reach. They use `--config-dir` to point at an isolated temp dir so they
//! never touch the developer's real `~/.config/rw`.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// A clean config dir with no profiles, no auth, no version-check cache.
fn empty_config_dir() -> TempDir {
    TempDir::new().unwrap()
}

/// Returns a `Command` configured with `--config-dir <dir>` and a stale
/// version-check cache so the binary doesn't make a live GitHub request.
fn rw(dir: &Path) -> Command {
    // Pre-seed a fresh version_check cache so `check_and_update` short-circuits
    // without going to GitHub. The version we store equals `CARGO_PKG_VERSION`,
    // so `is_newer` returns false and no warning is emitted.
    let cache_path = dir.join("version_check.json");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let cache = serde_json::json!({
        "checked_at": now,
        "latest_version": env!("CARGO_PKG_VERSION"),
    });
    std::fs::write(&cache_path, cache.to_string()).unwrap();

    let mut cmd = Command::cargo_bin("rw").unwrap();
    cmd.args(["--config-dir", dir.to_str().unwrap()]);
    cmd
}

#[test]
fn test_version_flag_succeeds() {
    let mut cmd = Command::cargo_bin("rw").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help_flag_succeeds() {
    let mut cmd = Command::cargo_bin("rw").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("RoundingWell CLI"));
}

#[test]
fn test_unknown_subcommand_fails() {
    let mut cmd = Command::cargo_bin("rw").unwrap();
    cmd.arg("nonexistent-subcommand").assert().failure().stderr(
        predicate::str::contains("unrecognized subcommand")
            .or(predicate::str::contains("invalid value")
                .or(predicate::str::contains("unexpected"))),
    );
}

#[test]
fn test_missing_subcommand_fails() {
    let mut cmd = Command::cargo_bin("rw").unwrap();
    cmd.assert().failure();
}

#[test]
fn test_invalid_profile_slug_rejected() {
    let dir = empty_config_dir();
    rw(dir.path())
        .args(["--profile", "Bad-Slug", "auth", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a valid slug"));
}

#[test]
fn test_no_profile_configured_errors_clearly() {
    let dir = empty_config_dir();
    rw(dir.path())
        .args(["teams", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no profile selected"));
}

#[test]
fn test_config_dir_must_exist() {
    let mut cmd = Command::cargo_bin("rw").unwrap();
    cmd.args(["--config-dir", "/definitely/not/a/real/path/zzz"])
        .args(["auth", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("config directory does not exist"));
}

#[test]
fn test_config_profile_list_on_empty_config() {
    let dir = empty_config_dir();
    rw(dir.path())
        .args(["config", "profile", "list"])
        .assert()
        .success();
}

#[test]
fn test_auth_login_with_json_flag_rejected() {
    // `auth login` is interactive and must refuse `--json`.
    let dir = empty_config_dir();
    // Set up a default profile so we get past profile resolution.
    let cfg = dir.path().join("config.json");
    std::fs::write(
        &cfg,
        r#"{"default":"demo","profiles":{"demo":{"organization":"demonstration","stage":"prod"}}}"#,
    )
    .unwrap();

    rw(dir.path())
        .args(["--json", "auth", "login"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("interactive"));
}

#[test]
fn test_auth_flag_with_login_rejected() {
    let dir = empty_config_dir();
    let cfg = dir.path().join("config.json");
    std::fs::write(
        &cfg,
        r#"{"default":"demo","profiles":{"demo":{"organization":"demonstration","stage":"prod"}}}"#,
    )
    .unwrap();

    rw(dir.path())
        .args(["--auth", "demo", "auth", "login"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--auth"));
}

#[test]
fn test_auth_status_unauthenticated_succeeds_with_message() {
    let dir = empty_config_dir();
    let cfg = dir.path().join("config.json");
    std::fs::write(
        &cfg,
        r#"{"default":"demo","profiles":{"demo":{"organization":"demonstration","stage":"prod"}}}"#,
    )
    .unwrap();

    rw(dir.path())
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Not authenticated"));
}
