#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

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
