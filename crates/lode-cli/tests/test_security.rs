#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn check_reports_convention_violations_with_exit_code_2() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("BadName.rs"), "").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .arg("check")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("BadName.rs"));
}

#[test]
fn check_fix_renames_convention_violations() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("BadName.rs"), "").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["check", "--fix"])
        .assert()
        .success()
        .stdout(predicate::str::contains("renamed"));

    assert!(temp.path().join("bad_name.rs").exists());
}

#[test]
fn scan_secrets_returns_exit_code_7() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("config.rs"), "API_KEY=real-value\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["scan", "secrets"])
        .assert()
        .code(7)
        .stdout(predicate::str::contains("suspicious credential assignment"));
}

#[test]
fn scan_secrets_quiet_supports_staged_flag() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("config.rs"), "API_KEY=real-value\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["scan", "secrets", "--quiet", "--staged"])
        .assert()
        .code(7)
        .stdout(predicate::str::contains("staged"));
}

#[test]
fn scan_foreign_reports_migration_actions() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("package.json"), "{}\n").expect("write file");
    std::fs::write(temp.path().join("BadName.TXT"), "hello\n").expect("write file");

    lode()
        .args(["scan", "foreign"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Foreign Project Scan"))
        .stdout(predicate::str::contains("npm"))
        .stdout(predicate::str::contains("package.json"))
        .stdout(predicate::str::contains("action run lode init"));
}

#[test]
fn scan_foreign_supports_json() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("go.mod"), "module example.com/demo\n").expect("write file");

    lode()
        .args(["scan", "foreign", "--output", "json"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"package_manager\": \"go\""))
        .stdout(predicate::str::contains("\"lode_project\": false"))
        .stdout(predicate::str::contains("\"migration_actions\""));
}

#[test]
fn mcp_lists_tools_resources_and_prompts() {
    lode()
        .args(["mcp", "--list-tools", "--list-resources", "--list-prompts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"tools\""))
        .stdout(predicate::str::contains("lode_config_show"))
        .stdout(predicate::str::contains("lode_scan_foreign"))
        .stdout(predicate::str::contains("lode_pkg_audit"))
        .stdout(predicate::str::contains("lode://config"))
        .stdout(predicate::str::contains("lode://project/info"))
        .stdout(predicate::str::contains("lode-project-review"));
}

#[test]
fn mcp_stdio_handles_tool_calls() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("package.json"), "{}\n").expect("write file");
    let commands = temp.path().join(".lode").join("commands");
    std::fs::create_dir_all(&commands).expect("create directory");
    std::fs::write(
        commands.join("deploy.toml"),
        "description = \"Deploy preview\"\n[[steps]]\nkind = \"lode\"\nrun = \"audit\"\n",
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .write_stdin(format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lode_template_list","arguments":{}}}"#,
            format!(
                r#"{{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{{"name":"lode_scan_foreign","arguments":{{"path":"{}"}}}}}}"#,
                temp.path().to_string_lossy().replace('\\', "\\\\")
            ),
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"lode_pkg_audit","arguments":{"fail_on":"high"}}}"#,
            r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"lode_custom_deploy","arguments":{}}}"#,
            r#"{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"lode://project/info"}}"#
        ))
        .arg("mcp")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"serverInfo\""))
        .stdout(predicate::str::contains("\"tools\""))
        .stdout(predicate::str::contains("lode_custom_deploy"))
        .stdout(predicate::str::contains("root/README.md"))
        .stdout(predicate::str::contains("package_manager"))
        .stdout(predicate::str::contains("migration_actions"))
        .stdout(predicate::str::contains("operation"))
        .stdout(predicate::str::contains("--audit-level"))
        .stdout(predicate::str::contains("slug"))
        .stdout(predicate::str::contains("Deploy preview"))
        .stdout(predicate::str::contains("lode://project/info"));
}

#[test]
fn lsp_stdio_handles_initialize_and_diagnostics() {
    lode()
        .write_stdin(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///demo/src/main.rs","text":"fn main() {\n let token = \"ghp_secret\";\n}\n"}}}
"#,
        )
        .args(["lsp", "--stdio"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lode-lsp"))
        .stdout(predicate::str::contains("textDocument/publishDiagnostics"))
        .stdout(predicate::str::contains("missing a lode signature"))
        .stdout(predicate::str::contains("possible secret token"));

    lode()
        .args(["lsp", "--capabilities"])
        .assert()
        .success()
        .stdout(predicate::str::contains("textDocumentSync"));
}
