#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn pkg_list_detects_package_manager() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("package.json"),
        r#"{"dependencies":{"react":"^18.0.0"},"devDependencies":{"vite":"^5.0.0"}}"#,
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["pkg", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("manager: npm"))
        .stdout(predicate::str::contains("package.json"))
        .stdout(predicate::str::contains("react ^18.0.0"));

    lode()
        .current_dir(temp.path())
        .args(["pkg", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"manager\": \"npm\""))
        .stdout(predicate::str::contains("\"name\": \"react\""))
        .stdout(predicate::str::contains("\"scope\": \"devDependencies\""));
}

#[test]
fn pkg_update_dry_run_prints_manager_command() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("package.json"), "{}\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["pkg", "update", "left-pad", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: npm update left-pad"));
}

#[test]
fn pkg_dry_run_translates_native_commands() {
    let node = tempfile::tempdir().expect("create temp dir");
    std::fs::write(node.path().join("pnpm-lock.yaml"), "lockfileVersion: '9'\n")
        .expect("write file");

    lode()
        .current_dir(node.path())
        .args(["pkg", "outdated", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: pnpm outdated"));

    lode()
        .current_dir(node.path())
        .args(["pkg", "outdated", "--dry-run", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"outdated\""))
        .stdout(predicate::str::contains("\"manager\": \"pnpm\""))
        .stdout(predicate::str::contains("\"command\": \"pnpm\""))
        .stdout(predicate::str::contains("\"outdated\""));

    lode()
        .current_dir(node.path())
        .args(["pkg", "why", "react", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: pnpm why react"));

    let python = tempfile::tempdir().expect("create temp dir");
    std::fs::write(python.path().join("requirements.txt"), "requests==2.0.0\n")
        .expect("write file");

    lode()
        .current_dir(python.path())
        .args(["pkg", "info", "requests", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "project -> requirements.txt -> requests ==2.0.0",
        ))
        .stdout(predicate::str::contains("would run: pip show requests"));

    lode()
        .current_dir(python.path())
        .args(["pkg", "info", "requests", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"query\": \"requests\""))
        .stdout(predicate::str::contains(
            "\"manifest\": \"requirements.txt\"",
        ));

    let go = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        go.path().join("go.mod"),
        "module demo\n\nrequire (\n  example.com/mod v1.0.0\n)\n",
    )
    .expect("write file");

    lode()
        .current_dir(go.path())
        .args(["pkg", "audit", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: go vulncheck ./..."))
        .stdout(predicate::str::contains("would run: lode scan secrets"));

    let cargo = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        cargo.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n",
    )
    .expect("write file");

    lode()
        .current_dir(cargo.path())
        .args(["pkg", "why", "serde", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"why\""))
        .stdout(predicate::str::contains("\"name\": \"serde\""))
        .stdout(predicate::str::contains("\"version\": \"1\""));

    lode()
        .current_dir(cargo.path())
        .args([
            "pkg",
            "audit",
            "--dry-run",
            "--format",
            "json",
            "--fail-on",
            "high",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"audit\""))
        .stdout(predicate::str::contains("\"manager\": \"cargo\""))
        .stdout(predicate::str::contains("\"--deny\""))
        .stdout(predicate::str::contains("\"high\""));

    let gradle = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        gradle.path().join("build.gradle"),
        "plugins { id 'java' }\ndependencies { testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0' }\n",
    )
    .expect("write file");

    lode()
        .current_dir(gradle.path())
        .args(["pkg", "why", "junit", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "project -> build.gradle -> org.junit.jupiter:junit-jupiter 5.10.0",
        ))
        .stdout(predicate::str::contains(
            "would run: gradle dependencyInsight --dependency junit",
        ));

    lode()
        .current_dir(gradle.path())
        .args(["pkg", "audit", "--dry-run", "--fail-on", "critical"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would run: gradle dependencyCheckAnalyze -DfailBuildOnCVSS=9",
        ));

    lode()
        .current_dir(gradle.path())
        .args(["pkg", "audit", "--dry-run", "--fail-on", "severe"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported package audit severity: severe",
        ));

    let maven = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        maven.path().join("pom.xml"),
        "<project><dependencies><dependency><groupId>org.junit.jupiter</groupId><artifactId>junit-jupiter</artifactId><version>5.10.0</version><scope>test</scope></dependency></dependencies></project>\n",
    )
    .expect("write file");

    lode()
        .current_dir(maven.path())
        .args(["pkg", "info", "junit-jupiter", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"name\": \"org.junit.jupiter:junit-jupiter\"",
        ))
        .stdout(predicate::str::contains("\"scope\": \"test\""));

    lode()
        .current_dir(maven.path())
        .args(["pkg", "update", "org.junit.jupiter:junit-jupiter", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would run: mvn versions:use-latest-releases -DgenerateBackupPoms=false -Dincludes=org.junit.jupiter:junit-jupiter",
        ));
}

#[test]
fn pkg_graph_json_reports_manifest() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname='x'\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["pkg", "graph", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"manager\": \"cargo\""))
        .stdout(predicate::str::contains("\"kind\": \"cargo\""))
        .stdout(predicate::str::contains("\"edges\""));
}

#[test]
fn pkg_graph_dot_reports_manifest() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("package.json"), "{}\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["pkg", "graph", "--format", "dot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph packages"))
        .stdout(predicate::str::contains("manager=npm"))
        .stdout(predicate::str::contains("project -> node"));
}
