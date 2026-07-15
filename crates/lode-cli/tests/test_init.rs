#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn setup_defaults_creates_config_and_dirs() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .arg("--defaults")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup"));

    assert!(temp.path().join(".lode").join("config.toml").exists());
    assert!(temp.path().join(".lode").join("templates").exists());
    assert!(temp.path().join(".lode").join("profiles").exists());
    assert!(temp.path().join(".lode").join("commands").exists());
    assert!(temp
        .path()
        .join(".lode")
        .join("profiles")
        .join("systems")
        .join("rust-cli.toml")
        .exists());
    assert!(temp
        .path()
        .join(".lode")
        .join("templates")
        .join("root")
        .join("README.md")
        .exists());
    assert!(temp
        .path()
        .join(".lode")
        .join("licenses")
        .join("GPL-3.0-only.txt")
        .exists());
    assert!(std::fs::read_to_string(
        temp.path()
            .join(".lode")
            .join("licenses")
            .join("MPL-2.0.txt")
    )
    .expect("read file")
    .contains("Mozilla Public License"));
}

#[test]
fn setup_and_export_respect_template_dir_override() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let templates = temp.path().join("custom-templates");
    let pack = temp.path().join("override.lodepack");

    lode()
        .env("LODE_CONFIG", &config)
        .env("LODE_TEMPLATES", &templates)
        .arg("setup")
        .assert()
        .success();

    assert!(templates.join("root").join("README.md").exists());
    assert!(!temp
        .path()
        .join(".lode")
        .join("templates")
        .join("root")
        .join("README.md")
        .exists());

    lode()
        .env("LODE_CONFIG", &config)
        .env("LODE_TEMPLATES", &templates)
        .args(["export", "--out"])
        .arg(&pack)
        .assert()
        .success();

    assert!(std::fs::read_to_string(pack)
        .expect("read file")
        .contains("templates/root/README.md"));
}

#[test]
fn init_dry_run_writes_nothing() {
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
        .arg("init")
        .arg("my-app")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));

    assert!(!temp.path().join("my-app").exists());
}

#[test]
fn new_alias_dry_run_matches_init() {
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
        .args(["new", "alias-app", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"))
        .stdout(predicate::str::contains("alias-app"));

    assert!(!temp.path().join("alias-app").exists());
}

#[test]
fn init_creates_default_project_layout() {
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
        .arg("init")
        .arg("my-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("initialised"));

    let project = temp.path().join("my-app");
    assert!(project.join(".lode").join("project.toml").exists());
    assert!(project.join("src").exists());
    assert!(project.join("tests").exists());
    assert!(project.join("_ref_").exists());
    assert!(project.join("_ctx_").exists());
    assert!(project.join("README.md").exists());
    assert!(project.join("Makefile").exists());
    assert!(project.join("AGENTS.md").exists());
    assert!(project
        .join(".lode")
        .join("context")
        .join("PLAN.md")
        .exists());
    assert!(std::fs::read_to_string(project.join(".gitignore"))
        .expect("read file")
        .contains(".env"));
    assert!(project.join(".git").exists());
}

#[test]
fn init_with_rust_profile_creates_language_files() {
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
        .args([
            "init",
            "cli-tool",
            "--profile",
            "systems/rust-cli",
            "--with",
            "ci,vscode",
        ])
        .assert()
        .success();

    let project = temp.path().join("cli-tool");
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("rust-toolchain.toml").exists());
    assert!(project
        .join(".github")
        .join("workflows")
        .join("ci.yml")
        .exists());
    assert!(project.join(".vscode").join("settings.json").exists());
    assert!(
        std::fs::read_to_string(project.join(".vscode").join("settings.json"))
            .expect("read file")
            .contains("lode.startDaemonOnOpen")
    );
    assert!(
        std::fs::read_to_string(project.join(".vscode").join("tasks.json"))
            .expect("read file")
            .contains("lode: release rollback preview")
    );
    assert!(
        std::fs::read_to_string(project.join(".vscode").join("launch.json"))
            .expect("read file")
            .contains("lode lsp --stdio")
    );
    let vscode_tasks: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(project.join(".vscode/tasks.json")).expect("read file"),
    )
    .expect("parse JSON");
    assert_eq!(vscode_tasks["version"], "2.0.0");
}

#[test]
fn add_editor_integrations_scaffolds_files() {
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
        .args(["init", "editor-app"])
        .assert()
        .success();

    let project = temp.path().join("editor-app");
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["add", "zed"])
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["add", "nvim"])
        .assert()
        .success();

    let zed_settings =
        std::fs::read_to_string(project.join(".zed").join("settings.json")).expect("read file");
    let zed_settings: serde_json::Value = serde_json::from_str(&zed_settings).expect("parse JSON");
    assert_eq!(zed_settings["lode"]["binary"], "lode");
    assert_eq!(zed_settings["lode"]["mcp_port"], 3847);

    let zed_tasks =
        std::fs::read_to_string(project.join(".zed").join("tasks.json")).expect("read file");
    let zed_tasks: serde_json::Value = serde_json::from_str(&zed_tasks).expect("parse JSON");
    assert!(zed_tasks
        .as_array()
        .expect("array expected")
        .iter()
        .any(|task| task["label"] == "lode: release rollback preview"));
    assert!(zed_tasks
        .as_array()
        .expect("array expected")
        .iter()
        .any(|task| task["label"] == "lode: daemon status"));
    assert!(std::fs::read_to_string(
        project
            .join(".config")
            .join("nvim")
            .join("lua")
            .join("lode.lua")
    )
    .expect("read file")
    .contains("configs.lode_lsp"));
    assert!(std::fs::read_to_string(
        project
            .join(".config")
            .join("nvim")
            .join("lua")
            .join("lode.lua")
    )
    .expect("read file")
    .contains("LodeRelease"));
    assert!(std::fs::read_to_string(
        project
            .join(".config")
            .join("nvim")
            .join("lua")
            .join("lode")
            .join("snippets.lua")
    )
    .expect("read file")
    .contains("lode_header"));
}

#[test]
fn rerunning_init_returns_exists_code() {
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
        .arg("init")
        .arg("my-app")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .arg("init")
        .arg("my-app")
        .assert()
        .code(4)
        .stderr(predicate::str::contains("already initialised"));
}

#[test]
fn init_registers_project_and_projects_list_shows_it() {
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
        .args(["init", "registered-app", "--profile", "core/app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("registered"));

    lode()
        .env("LODE_CONFIG", &config)
        .arg("projects")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("registered-app"))
        .stdout(predicate::str::contains("core/app"));
}

#[test]
fn init_uses_active_profile_when_profile_flag_is_absent() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .args(["profile", "use", "systems/rust-cli"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["init", "from-active-profile"])
        .assert()
        .success();

    assert!(temp
        .path()
        .join("from-active-profile")
        .join("Cargo.toml")
        .exists());
}

#[test]
fn cp_new_creates_problem_file() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["cp", "new", "b", "--lang", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created competitive problem b"));

    assert!(temp
        .path()
        .join("problems")
        .join("b")
        .join("main.rs")
        .exists());
}
