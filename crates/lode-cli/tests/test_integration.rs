#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn export_import_round_trips_lodepack() {
    let source = tempfile::tempdir().expect("create temp dir");
    let source_config = isolated_config(&source);
    let pack = source.path().join("setup.lodepack");

    lode()
        .env("LODE_CONFIG", &source_config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &source_config)
        .args(["config", "set", "identity.author", "Exported User"])
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &source_config)
        .arg("export")
        .arg("--out")
        .arg(&pack)
        .assert()
        .success()
        .stdout(predicate::str::contains("exported"));

    let raw_pack: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&pack).expect("read file"))
            .expect("parse JSON");
    assert_eq!(raw_pack["manifest"]["schema_version"], 3);
    assert_eq!(
        raw_pack["manifest"]["checksum_algorithm"],
        "lode-default-hash-v1"
    );
    assert!(
        raw_pack["manifest"]["file_count"]
            .as_u64()
            .expect("u64 expected")
            > 0
    );
    assert!(raw_pack["files"]
        .as_array()
        .expect("array expected")
        .iter()
        .all(|file| file["checksum"].as_str().unwrap_or_default().len() == 64));

    let dest = tempfile::tempdir().expect("create temp dir");
    let dest_config = isolated_config(&dest);
    lode()
        .env("LODE_CONFIG", &dest_config)
        .arg("import")
        .arg(&pack)
        .assert()
        .success()
        .stdout(predicate::str::contains("imported"));

    assert!(
        std::fs::read_to_string(dest.path().join(".lode").join("config.toml"))
            .expect("read file")
            .contains("Exported User")
    );
    assert!(dest
        .path()
        .join(".lode")
        .join("templates")
        .join("root")
        .join("README.md")
        .exists());
}

#[test]
fn export_filters_and_import_conflict_modes_work() {
    let source = tempfile::tempdir().expect("create temp dir");
    let source_config = isolated_config(&source);
    let pack = source.path().join("filtered.lodepack");

    lode()
        .env("LODE_CONFIG", &source_config)
        .arg("setup")
        .assert()
        .success();
    std::fs::write(
        source.path().join(".lode").join("registry.json"),
        "{\"projects\":[]}",
    )
    .expect("write file");
    lode()
        .env("LODE_CONFIG", &source_config)
        .args([
            "export",
            "--no-templates",
            "--no-snippets",
            "--no-commands",
            "--include-metrics",
            "--out",
        ])
        .arg(&pack)
        .assert()
        .success();

    let raw = std::fs::read_to_string(&pack).expect("read file");
    assert!(!raw.contains("templates/root/README.md"));
    assert!(!raw.contains("snippets/rs/serde-struct.snippet"));
    assert!(!raw.contains("commands/health.toml"));
    assert!(raw.contains("registry.json"));

    let dest = tempfile::tempdir().expect("create temp dir");
    let dest_config = isolated_config(&dest);
    lode()
        .env("LODE_CONFIG", &dest_config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &dest_config)
        .args(["import", "--no-merge"])
        .arg(&pack)
        .assert()
        .failure()
        .stderr(predicate::str::contains("import conflict"));
    lode()
        .env("LODE_CONFIG", &dest_config)
        .args(["import", "--force"])
        .arg(&pack)
        .assert()
        .success();
}

#[test]
fn import_rejects_tampered_lodepack_contents() {
    let source = tempfile::tempdir().expect("create temp dir");
    let source_config = isolated_config(&source);
    let pack = source.path().join("tampered.lodepack");

    lode()
        .env("LODE_CONFIG", &source_config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &source_config)
        .arg("export")
        .arg("--out")
        .arg(&pack)
        .assert()
        .success();

    let mut raw_pack: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&pack).expect("read file"))
            .expect("parse JSON");
    let config_file = raw_pack["files"]
        .as_array_mut()
        .expect("array expected")
        .iter_mut()
        .find(|file| file["path"] == "config.toml")
        .expect("array expected");
    config_file["contents"] =
        serde_json::Value::String("schema_version = 3\n# tampered\n".to_string());
    std::fs::write(
        &pack,
        serde_json::to_string_pretty(&raw_pack).expect("serialize JSON"),
    )
    .expect("write file");

    let dest = tempfile::tempdir().expect("create temp dir");
    let dest_config = isolated_config(&dest);
    lode()
        .env("LODE_CONFIG", &dest_config)
        .arg("import")
        .arg(&pack)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "lodepack checksum mismatch for config.toml",
        ));

    assert!(!dest.path().join(".lode").join("config.toml").exists());
}

#[test]
fn import_rejects_unsafe_lodepack_paths() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let pack = temp.path().join("unsafe.lodepack");
    std::fs::write(
        &pack,
        r#"{
  "version": 1,
  "files": [
    { "path": "../outside.txt", "contents": "bad" }
  ]
}"#,
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("import")
        .arg(&pack)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsafe lodepack path"));

    assert!(!temp.path().join("outside.txt").exists());
}

#[test]
fn import_rejects_duplicate_lodepack_paths() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let pack = temp.path().join("duplicate.lodepack");
    std::fs::write(
        &pack,
        r#"{
  "version": 1,
  "files": [
    { "path": "config.toml", "contents": "schema_version = 3\n" },
    { "path": "config.toml", "contents": "schema_version = 3\n[identity]\nauthor = \"shadow\"\n" }
  ]
}"#,
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("import")
        .arg(&pack)
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate lodepack path"));

    assert!(!temp.path().join(".lode").join("config.toml").exists());
}

#[test]
fn agent_sync_plan_and_export_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(temp.path().join("_ref_")).expect("create directory");
    std::fs::create_dir_all(temp.path().join("_ctx_")).expect("create directory");
    std::fs::write(
        temp.path().join("_ref_").join("ARCHITECTURE.md"),
        "# Arch\n",
    )
    .expect("write file");
    std::fs::write(temp.path().join("AGENTS.md"), "# Agent\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["agent", "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent context synced"));

    lode()
        .current_dir(temp.path())
        .args(["agent", "plan", "init"])
        .assert()
        .success();
    lode()
        .current_dir(temp.path())
        .args([
            "agent",
            "plan",
            "add",
            "finish daemon",
            "--branch",
            "feat/daemon",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("added task 1"));
    lode()
        .current_dir(temp.path())
        .args(["agent", "plan", "done", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completed task 1"));
    lode()
        .current_dir(temp.path())
        .args(["agent", "plan", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("done"))
        .stdout(predicate::str::contains("finish daemon"));

    let out = temp.path().join("agent.lodepack");
    lode()
        .current_dir(temp.path())
        .args(["agent", "export", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::contains("exported agent context"));
    assert!(out.exists());
}

#[test]
fn sync_refreshes_agent_context_and_supports_dry_run() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    std::fs::create_dir_all(temp.path().join("_ref_")).expect("create directory");
    std::fs::create_dir_all(temp.path().join("_ctx_")).expect("create directory");
    std::fs::write(temp.path().join("_ctx_").join("NOTES.md"), "# Notes\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["sync", "--dry-run", "--section", "agent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would sync agent"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["sync", "--section", "agent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent context synced"));

    assert!(temp
        .path()
        .join(".lode")
        .join("context")
        .join("INDEX.md")
        .exists());
}

#[test]
fn sync_templates_reconciles_project_scaffold() {
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
        .args(["init", "my-app"])
        .assert()
        .success();

    let project = temp.path().join("my-app");
    assert!(project.join(".lode").join("scaffold.lock").exists());
    std::fs::remove_file(project.join("README.md")).expect("remove file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["sync", "--dry-run", "--section", "templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would reconcile"));

    assert!(!project.join("README.md").exists());

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["sync", "--section", "templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("synced"));

    assert!(project.join("README.md").exists());
}

#[test]
fn health_writes_metrics_and_metrics_show_reads_them() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    std::fs::write(temp.path().join("README.md"), "# App\n").expect("write file");
    std::fs::write(temp.path().join("LICENSE"), "MIT\n").expect("write file");
    std::fs::write(temp.path().join(".env.example"), "APP_NAME=app\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("health score"));

    assert!(temp.path().join(".lode").join("metrics.json").exists());

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["metrics", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("metrics score"));
}

#[test]
fn metrics_baseline_trend_and_diff_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("README.md"), "# App\n").expect("write file");
    std::fs::write(temp.path().join("LICENSE"), "MIT\n").expect("write file");
    std::fs::write(temp.path().join(".env.example"), "APP_NAME=app\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["metrics", "baseline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("metrics baseline saved"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .arg("health")
        .assert()
        .success();

    lode()
        .current_dir(temp.path())
        .args(["metrics", "trend", "--last", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("last 3 snapshot"));

    lode()
        .current_dir(temp.path())
        .args(["metrics", "diff-baseline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("score delta"));
}

#[test]
fn daemon_start_status_log_and_stop_are_stateful() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "start"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon started"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("active"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "list-watchers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rename\tactive"))
        .stdout(predicate::str::contains("env_drift\tactive"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "pause"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon paused"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "status", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("paused"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "list-watchers", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"paused\": true"))
        .stdout(predicate::str::contains("\"watchers\""));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "resume"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon resumed"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "log"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon started"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "stop"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon stopped"));
}

#[test]
fn daemon_flags_status_json_and_log_tail_are_stateful() {
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
        .args(["daemon", "start", "--no-rename", "--foreground"])
        .assert()
        .success()
        .stdout(predicate::str::contains("foreground daemon watching"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["daemon", "status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"active\":true"))
        .stdout(predicate::str::contains("\"watchers\""))
        .stdout(predicate::str::contains("headers"));

    let state = std::fs::read_to_string(
        temp.path()
            .join(".lode")
            .join("cache")
            .join("daemon-state.json"),
    )
    .expect("read file");
    assert!(state.contains("\"events\""));
    assert!(state.contains("\"foreground\""));

    let project_state =
        std::fs::read_to_string(temp.path().join(".lode").join("daemon-state.json"))
            .expect("read file");
    assert!(project_state.contains("\"schema_version\": 3"));
    assert!(project_state.contains("\"file_count\""));
    assert!(project_state.contains("\"content_hash\""));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "log", "--tail", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("foreground daemon exited"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "stop", "--project", "demo"])
        .assert()
        .success();
}

#[test]
fn daemon_status_json_includes_recent_events() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "start"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"recent_events\":[{"))
        .stdout(predicate::str::contains(
            "\"message\":\"daemon started foreground=false rename=true sign=true stamp=true\"",
        ));
}

#[test]
fn daemon_log_follow_can_exit_without_polling() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "start"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .env("LODE_DAEMON_FOLLOW_TICKS", "0")
        .args(["daemon", "log", "--follow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon started"));
}

#[test]
fn serve_renders_dashboard_snapshot() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    std::fs::write(temp.path().join("README.md"), "# App\n").expect("write file");
    std::fs::write(temp.path().join("LICENSE"), "MIT\n").expect("write file");
    std::fs::write(temp.path().join(".env.example"), "APP_NAME=app\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["serve", "--no-color", "--no-live", "--pane", "metrics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lode serve"))
        .stdout(predicate::str::contains("Pane: metrics"))
        .stdout(predicate::str::contains("PROJECT HEALTH"))
        .stdout(predicate::str::contains("CROSS-PROJECT REGISTRY"));
}
