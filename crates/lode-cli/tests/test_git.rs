#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn git_branch_formats_slug() {
    lode()
        .args(["git", "branch", "feat", "User Login Flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feat/user-login-flow"));
}

#[test]
fn git_setup_commands_write_metadata_files() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["git", "sign-setup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("git signing setup recorded"));

    lode()
        .current_dir(temp.path())
        .args([
            "git",
            "remote-setup",
            "--provider",
            "github",
            "--visibility",
            "public",
            "--token-env",
            "GH_TOKEN",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("git remote setup recorded"));

    assert!(temp.path().join(".lode").join("git-signing.toml").exists());
    assert!(
        std::fs::read_to_string(temp.path().join(".lode").join("remote.toml"))
            .expect("read file")
            .contains("GH_TOKEN")
    );
}

#[test]
fn git_changelog_can_write_plain_output() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let out = temp.path().join("CHANGELOG.generated");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .status()
        .expect("command should succeed");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .status()
        .expect("command should succeed");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp.path())
        .status()
        .expect("command should succeed");
    std::fs::write(temp.path().join("file.txt"), "hello\n").expect("write file");
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .status()
        .expect("command should succeed");
    std::process::Command::new("git")
        .args(["commit", "-m", "feat: hello"])
        .current_dir(temp.path())
        .status()
        .expect("command should succeed");

    lode()
        .current_dir(temp.path())
        .args(["git", "changelog", "--format", "plain", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote changelog"));

    assert!(std::fs::read_to_string(out)
        .expect("read file")
        .contains("feat: hello"));
}

#[test]
fn gha_add_and_validate_workflows() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["gha", "add", "ci-rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("added workflow ci-rust"));

    lode()
        .current_dir(temp.path())
        .args(["gha", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("validated 1 workflow"));
}
