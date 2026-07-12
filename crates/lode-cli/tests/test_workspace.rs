#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn workspace_init_add_list_and_graph_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["workspace", "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace initialised"));

    lode()
        .current_dir(temp.path())
        .args(["workspace", "add", "crates/app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace member added"));
    lode()
        .current_dir(temp.path())
        .args(["workspace", "add", "crates/lib"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace member added"));

    lode()
        .current_dir(temp.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("crates/app"));

    lode()
        .current_dir(temp.path())
        .args(["workspace", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"crates/app\""));

    lode()
        .current_dir(temp.path())
        .args(["workspace", "graph"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-> crates/app"));

    lode()
        .current_dir(temp.path())
        .args(["workspace", "graph", "--format", "dot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph workspace"));

    lode()
        .current_dir(temp.path())
        .args([
            "workspace",
            "run",
            "test",
            "--pkg",
            "app",
            "--parallel",
            "2",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("parallel requested: 2"))
        .stdout(predicate::str::contains(
            "would run: make -C crates/app test",
        ));

    lode()
        .current_dir(temp.path())
        .args([
            "workspace",
            "run",
            "build",
            "--changed",
            "crates/lib/src/main.rs",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would run: make -C crates/lib build",
        ))
        .stdout(predicate::str::contains("crates/app").not());

    lode()
        .current_dir(temp.path())
        .args(["workspace", "remove", "crates/app", "--confirm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace member removed"));
}

#[test]
fn workspace_add_rejects_unsafe_member_path() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["workspace", "init"])
        .assert()
        .success();

    lode()
        .current_dir(temp.path())
        .args(["workspace", "add", "../outside"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsafe relative path"));
}
