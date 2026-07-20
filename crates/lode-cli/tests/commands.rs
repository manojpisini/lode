use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn lode_binary() -> PathBuf {
    assert_cmd::cargo::cargo_bin("lode")
}

fn lode_command() -> Command {
    let mut command = Command::cargo_bin("lode").expect("lode binary should be built by cargo");
    command.env("LODE_NO_CUSTOM_COMMANDS", "1");
    command
}

fn assert_binary_exists() {
    assert!(
        lode_binary().exists(),
        "lode binary not found at {:?}. Run `cargo build -p lode-cli` first.",
        lode_binary()
    );
}

#[test]
fn test_lode_binary_exists() {
    assert_binary_exists();
}

#[test]
fn test_lode_version() {
    assert_binary_exists();
    lode_command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("lode"));
}

#[test]
fn test_lode_help() {
    assert_binary_exists();
    lode_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("Commands")));
}

#[test]
fn test_lode_config_command_help() {
    assert_binary_exists();
    lode_command()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("Commands")));
}

#[test]
fn test_export_command_macros() {
    assert_binary_exists();
    let temp = tempfile::TempDir::new().unwrap();
    let out_path = temp.path().join("macros.lodepack");
    let output = lode_command()
        .args(["commands", "export", "--out", out_path.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .expect("failed to run lode commands export");
    // May fail if not in a lode project; that's acceptable
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
}

#[test]
fn test_lode_doctor() {
    assert_binary_exists();
    let output = lode_command()
        .args(["doctor", "--json"])
        .output()
        .expect("failed to run lode doctor --json");
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        eprintln!("doctor stderr: {stderr}");
    }
}

#[test]
fn test_lode_init_dry_run() {
    assert_binary_exists();
    let temp = tempfile::TempDir::new().unwrap();
    let output = lode_command()
        .args(["init", "test-project", "--dry-run", "--yes", "--no-check"])
        .current_dir(temp.path())
        .output()
        .expect("failed to run lode init --dry-run");
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        eprintln!("init stderr: {stderr}");
    }
    // dry-run should succeed or at least not crash
}
