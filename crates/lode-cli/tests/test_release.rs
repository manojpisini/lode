#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn release_bumps_cargo_version() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["release", "--bump", "patch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.1"));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read file")
        .contains("version = \"0.1.1\""));
    assert!(!temp
        .path()
        .join(".lode")
        .join("release.rollback.json")
        .exists());
}

#[test]
fn release_bumps_workspace_cargo_version() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"1.2.3\"\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["release", "--bump", "minor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.3.0"));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read file")
        .contains("version = \"1.3.0\""));
}

#[test]
fn release_dry_run_does_not_write() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("pyproject.toml"),
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["release", "--bump", "minor", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would update"));

    assert!(std::fs::read_to_string(temp.path().join("pyproject.toml"))
        .expect("read file")
        .contains("version = \"0.1.0\""));
}

#[test]
fn release_rollback_restores_pending_state() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let before = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
    let after = "[package]\nname = \"demo\"\nversion = \"0.1.1\"\n";
    std::fs::write(temp.path().join("Cargo.toml"), after).expect("write file");
    write_release_rollback(&temp, before, after);

    lode()
        .current_dir(temp.path())
        .args(["release", "--rollback", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would rollback Cargo.toml"));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read file")
        .contains("version = \"0.1.1\""));

    lode()
        .current_dir(temp.path())
        .args(["release", "--rollback"])
        .assert()
        .success()
        .stderr(predicate::str::contains("release rollback applied"));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read file")
        .contains("version = \"0.1.0\""));
    assert!(!temp
        .path()
        .join(".lode")
        .join("release.rollback.json")
        .exists());
}

#[test]
fn release_rollback_refuses_tampered_current_file() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let before = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
    let after = "[package]\nname = \"demo\"\nversion = \"0.1.1\"\n";
    let tampered = "[package]\nname = \"demo\"\nversion = \"0.1.2\"\n";
    std::fs::write(temp.path().join("Cargo.toml"), tampered).expect("write file");
    write_release_rollback(&temp, before, after);

    lode()
        .current_dir(temp.path())
        .args(["release", "--rollback"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "release rollback refused because Cargo.toml changed",
        ));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read file")
        .contains("version = \"0.1.2\""));
    assert!(temp
        .path()
        .join(".lode")
        .join("release.rollback.json")
        .exists());
}
