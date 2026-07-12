#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn config_show_defaults_prints_valid_toml() {
    lode()
        .arg("config")
        .arg("show")
        .arg("--defaults")
        .arg("--format")
        .arg("toml")
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_version = 3"));
}

#[test]
fn config_show_defaults_prints_valid_json() {
    lode()
        .arg("config")
        .arg("show")
        .arg("--defaults")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 3"));
}

#[test]
fn config_show_supports_section_filtering() {
    lode()
        .args([
            "config",
            "show",
            "--defaults",
            "--section",
            "identity",
            "--format",
            "toml",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("author"))
        .stdout(predicate::str::contains("license"))
        .stdout(predicate::str::contains("schema_version").not());

    lode()
        .args([
            "config",
            "show",
            "--defaults",
            "--section",
            "git",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"initial_branch\""));
}

#[test]
fn config_show_project_reads_project_config() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["init", "project-app"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path().join("project-app"))
        .args([
            "config",
            "show",
            "--project",
            "--section",
            "project",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"project-app\""))
        .stdout(predicate::str::contains("\"created_by\": \"lode\""));
}

#[test]
fn config_validate_supports_project_and_defaults() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["init", "project-app"])
        .assert()
        .success();

    lode()
        .args(["config", "validate", "--defaults"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default config valid"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path().join("project-app"))
        .args(["config", "validate", "--project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("project config valid"));
}

#[test]
fn config_validate_project_schema_mismatch_exits_schema() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["init", "project-app"])
        .assert()
        .success();

    let project_config = temp
        .path()
        .join("project-app")
        .join(".lode")
        .join("project.toml");
    let raw = std::fs::read_to_string(&project_config).expect("read file");
    std::fs::write(
        &project_config,
        raw.replacen("schema_version = 3", "schema_version = 2", 1),
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path().join("project-app"))
        .args(["config", "validate", "--project"])
        .assert()
        .code(6)
        .stderr(predicate::str::contains("schema version mismatch"));
}

#[test]
fn config_set_updates_global_config() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["config", "set", "identity.author", "Manoj"])
        .assert()
        .success()
        .stdout(predicate::str::contains("set identity.author"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("author = \"Manoj\""));
}

#[test]
fn config_diff_reports_changed_values() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .args(["config", "set", "identity.org", "acme"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["config", "diff"])
        .assert()
        .success()
        .stdout(predicate::str::contains("identity.org"));
}

#[test]
fn config_reset_restores_default_value() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .args(["config", "set", "identity.author", "Someone"])
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .args(["config", "reset", "identity.author"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reset identity.author"));

    assert!(std::fs::read_to_string(config)
        .expect("read file")
        .contains("author = \"Your Name\""));
}
