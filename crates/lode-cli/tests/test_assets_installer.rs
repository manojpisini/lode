#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, contents).expect("write file");
}

fn write_source_pack(source: &std::path::Path, template_body: &str) {
    write_file(&source.join("templates/app/main.txt"), template_body);
    write_file(&source.join("snippets/log.toml"), "body = 'log'\n");
    write_file(
        &source.join("lode.json"),
        r#"{
  "name": "local-pack",
  "description": "local test pack",
  "version": "0.1.0",
  "assets": [
    {"name":"app-template","type":"template","path":"templates/app","description":"app template"},
    {"name":"log-snippet","type":"snippet","path":"snippets/log.toml","description":"log snippet"}
  ]
}"#,
    );
}

#[test]
fn assets_add_list_remove_project_local_source() {
    let project = tempfile::tempdir().expect("create project temp");
    let source = tempfile::tempdir().expect("create source temp");
    write_source_pack(source.path(), "hello project");

    lode()
        .current_dir(project.path())
        .args([
            "assets",
            "add",
            source.path().to_str().expect("utf8 source"),
            "--all",
            "--project",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("installed 2 asset"));

    assert_eq!(
        std::fs::read_to_string(project.path().join(".lode/templates/app-template/main.txt"))
            .expect("read installed template"),
        "hello project"
    );
    assert!(project.path().join(".lode/snippets/log.toml").exists());

    lode()
        .current_dir(project.path())
        .args(["assets", "list", "--project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("app-template"))
        .stdout(predicate::str::contains("log-snippet"));

    lode()
        .current_dir(project.path())
        .args(["assets", "remove", "log-snippet", "--project", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed log-snippet"));
    assert!(!project.path().join(".lode/snippets/log.toml").exists());
}

#[test]
fn assets_add_selected_global_local_source() {
    let project = tempfile::tempdir().expect("create project temp");
    let source = tempfile::tempdir().expect("create source temp");
    write_source_pack(source.path(), "hello global");
    let config = isolated_config(&project);

    lode()
        .current_dir(project.path())
        .env("LODE_CONFIG", &config)
        .args([
            "assets",
            "add",
            source.path().to_str().expect("utf8 source"),
            "--asset",
            "app-template",
            "--global",
        ])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(project.path().join(".lode/templates/app-template/main.txt"))
            .expect("read global template"),
        "hello global"
    );

    lode()
        .current_dir(project.path())
        .env("LODE_CONFIG", &config)
        .args(["assets", "list", "--global", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"app-template\""));
}

#[test]
fn assets_duplicate_install_is_rejected() {
    let project = tempfile::tempdir().expect("create project temp");
    let source = tempfile::tempdir().expect("create source temp");
    write_source_pack(source.path(), "hello project");
    let source_arg = source.path().to_str().expect("utf8 source");

    lode()
        .current_dir(project.path())
        .args(["assets", "add", source_arg, "--asset", "app-template", "-p"])
        .assert()
        .success();

    lode()
        .current_dir(project.path())
        .args(["assets", "add", source_arg, "--asset", "app-template", "-p"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("asset already installed"));
}

#[test]
fn assets_update_replaces_installed_payload() {
    let project = tempfile::tempdir().expect("create project temp");
    let source = tempfile::tempdir().expect("create source temp");
    write_source_pack(source.path(), "v1");
    let source_arg = source.path().to_str().expect("utf8 source");

    lode()
        .current_dir(project.path())
        .args(["assets", "add", source_arg, "--asset", "app-template", "-p"])
        .assert()
        .success();

    write_source_pack(source.path(), "v2");
    lode()
        .current_dir(project.path())
        .args(["assets", "update", "app-template", "-p", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated 1 asset"));

    assert_eq!(
        std::fs::read_to_string(project.path().join(".lode/templates/app-template/main.txt"))
            .expect("read updated template"),
        "v2"
    );
}

#[test]
fn assets_init_creates_lode_json() {
    let project = tempfile::tempdir().expect("create project temp");
    lode()
        .current_dir(project.path())
        .args(["assets", "init", "starter-pack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote lode.json"));
    let raw = std::fs::read_to_string(project.path().join("lode.json")).expect("read manifest");
    assert!(raw.contains("starter-pack"));
}
