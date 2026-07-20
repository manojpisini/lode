#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn doctor_fix_reports_structured_checks() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("package.json"), "{}\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["doctor", "--fix", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"fixed\": true"))
        .stdout(predicate::str::contains("\"name\": \"config\""))
        .stdout(predicate::str::contains("\"name\": \"package_manager\""))
        .stdout(predicate::str::contains("\"detail\": \"npm\""))
        .stdout(predicate::str::contains("\"name\": \"upgrade\""));

    assert!(temp.path().join(".lode").join("config.toml").exists());
    assert!(temp.path().join(".lode").join("templates").exists());
}

#[test]
fn template_and_snippet_lists_support_json() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["template", "list", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("root/README.md"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "list", "--output", "json", "--lang", "rs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"serde-struct\""));
}

#[test]
fn rules_list_and_validate_use_config() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["rules", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default_case"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["rules", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rules valid"));
}

#[test]
fn sign_and_stamp_write_headers() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let file = temp.path().join("main.rs");
    std::fs::write(&file, "fn main() {}\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["sign"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("stamped"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["stamp", "--license"])
        .arg(&file)
        .assert()
        .success();

    let contents = std::fs::read_to_string(file).expect("read file");
    assert!(contents.contains("Generated with Lode"));
    assert!(contents.contains("MIT OR Apache-2.0"));
}

#[test]
fn projects_prune_removes_missing_projects() {
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
        .args(["init", "gone-app"])
        .assert()
        .success();

    std::fs::remove_dir_all(temp.path().join("gone-app")).expect("remove directory");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("projects")
        .arg("prune")
        .assert()
        .success()
        .stdout(predicate::str::contains("removed 1"));
}

#[test]
fn projects_cd_and_remove_use_registry() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let project = temp.path().join("manual-app");
    std::fs::create_dir_all(&project).expect("create directory");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["projects", "register"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["projects", "cd", "manual-app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("manual-app"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["projects", "remove", "manual-app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed project manual-app"));
}

#[test]
fn projects_and_license_lists_support_json() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let project = temp.path().join("json-app");
    std::fs::create_dir_all(&project).expect("create directory");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["projects", "register"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args([
            "projects",
            "list",
            "--output",
            "json",
            "--sort",
            "last-seen",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"json-app\""));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["license", "list", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MIT.txt"));
}

#[test]
fn projects_health_supports_json_and_refresh() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let project = temp.path().join("healthy-app");
    std::fs::create_dir_all(&project).expect("create directory");
    std::fs::write(project.join("README.md"), "# App\n").expect("write file");
    std::fs::write(project.join("LICENSE"), "MIT\n").expect("write file");
    std::fs::write(project.join(".env.example"), "APP_NAME=app\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["projects", "register"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["projects", "health", "--output", "json", "--refresh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"healthy-app\""))
        .stdout(predicate::str::contains("\"score\""));
}

#[test]
fn env_sync_and_check_use_env_example() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join(".env.example"),
        "APP_NAME=test\nLOG_LEVEL=debug\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["env", "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("added 2"));

    lode()
        .current_dir(temp.path())
        .args(["env", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("env ok"));
}

#[test]
fn env_add_updates_env_example() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["env", "add", "API_URL"])
        .assert()
        .success();

    assert!(std::fs::read_to_string(temp.path().join(".env.example"))
        .expect("read file")
        .contains("API_URL="));
}

#[test]
fn env_add_supports_default_comment_and_secret() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args([
            "env",
            "add",
            "API_TOKEN",
            "--default",
            "dev-token",
            "--comment",
            "Local API token",
            "--secret",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("added env key API_TOKEN"));

    let example = std::fs::read_to_string(temp.path().join(".env.example")).expect("read file");
    let env = std::fs::read_to_string(temp.path().join(".env")).expect("read file");
    assert!(example.contains("# Local API token"));
    assert!(example.contains("API_TOKEN=\n"));
    assert!(env.contains("API_TOKEN=dev-token"));
}

#[test]
fn license_set_and_check_write_license_file() {
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
        .args(["license", "set", "MIT"])
        .assert()
        .success()
        .stdout(predicate::str::contains("license set"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["license", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("license ok"));
}

#[test]
fn license_add_info_apply_and_remove_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["license", "add", "Custom-1.0", "--text", "Custom terms"])
        .assert()
        .success()
        .stdout(predicate::str::contains("added license Custom-1.0"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["license", "info", "Custom-1.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("id: Custom-1.0"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["license", "apply", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would apply license"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["license", "remove", "Custom-1.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed license Custom-1.0"));
}

#[test]
fn snippet_search_finds_extracted_snippets() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "search", "serde"])
        .assert()
        .success()
        .stdout(predicate::str::contains("serde-struct.snippet"));
}

#[test]
fn snippet_export_writes_vscode_json() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let out = temp.path().join("rust-snippets.json");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "export", "--lang", "rs", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::contains("exported"));

    let exported = std::fs::read_to_string(out).expect("read file");
    assert!(exported.contains("rs:serde-struct"));
    assert!(serde_json::from_str::<serde_json::Value>(&exported).is_ok());
}

#[test]
fn snippet_export_accepts_zed_format() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "export", "--lang", "rs", "--format", "zed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("serde-struct"));
}

#[test]
fn snippet_add_insert_and_remove_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let target = temp.path().join("notes.txt");
    std::fs::write(&target, "before\nafter\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args([
            "snippet",
            "add",
            "hello",
            "--lang",
            "txt",
            "--trigger",
            "hello world",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("created snippet txt/hello"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "insert", "hello", "--lang", "txt", "--line", "2"])
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("inserted snippet hello"));

    let contents = std::fs::read_to_string(&target).expect("read file");
    assert!(contents.contains("before\nhello world $1\nafter"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "remove", "hello", "--lang", "txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed snippet hello"));
}

#[test]
fn recipe_apply_writes_declared_files() {
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
        .args(["recipe", "apply", "docker-basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote"));

    assert!(temp.path().join("Dockerfile").exists());
}

#[test]
fn recipe_new_and_compose_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["recipe", "new", "my-stack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created recipe my-stack"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["recipe", "compose", "my-stack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote"));

    assert!(temp.path().join("docs").join("my-stack.md").exists());
}

#[test]
fn commands_run_dry_run_reads_extracted_macro() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["commands", "run", "verify", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verify"))
        .stdout(predicate::str::contains("Full verification pipeline"));
}

#[test]
fn commands_run_executes_project_local_shell_macro() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let commands_dir = temp.path().join(".lode").join("commands");
    std::fs::create_dir_all(&commands_dir).expect("create directory");
    std::fs::write(
        commands_dir.join("touch-file.toml"),
        "slug = \"touch-file\"\n\n[[steps]]\nkind = \"shell\"\nrun = \"whoami\"\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["commands", "run", "touch-file"])
        .assert()
        .success()
        .stdout(predicate::str::contains("whoami"));
}

#[test]
fn custom_command_direct_invocation_supports_help_and_dry_run() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let commands_dir = temp.path().join(".lode").join("commands");
    std::fs::create_dir_all(&commands_dir).expect("create directory");
    std::fs::write(
        commands_dir.join("deploy.toml"),
        "description = \"Deploy app\"\n\n[[steps]]\nkind = \"shell\"\nrun = \"echo deployed> deployed.txt\"\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["deploy", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploy app"))
        .stdout(predicate::str::contains("--dry-run"));

    lode()
        .current_dir(temp.path())
        .args(["deploy", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("echo deployed"));

    assert!(!temp.path().join("deployed.txt").exists());

    lode()
        .env("LODE_NO_CUSTOM_COMMANDS", "1")
        .current_dir(temp.path())
        .arg("deploy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("custom commands are disabled"));
}

#[test]
fn commands_add_export_and_remove_local_macro() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let exported = temp.path().join("commands.json");
    let exported_arg = std::path::Path::new("commands.json");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["commands", "add", "deploy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created command macro deploy"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["commands", "export", "--out"])
        .arg(exported_arg)
        .assert()
        .success()
        .stdout(predicate::str::contains("exported"));

    assert!(std::fs::read_to_string(&exported)
        .expect("read file")
        .contains("deploy.toml"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["commands", "remove", "deploy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed command macro deploy"));
}

#[test]
fn plugin_add_info_and_remove_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let source = temp.path().join("my-plugin");
    std::fs::create_dir_all(source.join("templates")).expect("create directory");
    std::fs::write(source.join("templates").join("README.md"), "# Plugin\n").expect("write file");
    std::fs::write(
        source.join("plugin.toml"),
        "[plugin]\nname = \"my-plugin\"\nversion = \"1.2.3\"\ndescription = \"Test plugin for local templates\"\n",
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "add"])
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("added plugin my-plugin"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "info", "my-plugin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("version\t1.2.3"))
        .stdout(predicate::str::contains("templates\tok"))
        .stdout(predicate::str::contains("trusted\tok"))
        .stdout(predicate::str::contains("installed_from"));

    let receipt = std::fs::read_to_string(
        temp.path()
            .join(".lode")
            .join("plugins")
            .join("my-plugin")
            .join(".lode-install.json"),
    )
    .expect("read file");
    assert!(receipt.contains("\"schema_version\": 3"));
    assert!(receipt.contains("\"reviewed\": true"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "search", "templates", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"my-plugin\""))
        .stdout(predicate::str::contains("\"installed\": true"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "remove", "my-plugin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed plugin my-plugin"));
}

#[test]
fn plugin_add_enforces_unsafe_permissions() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let source = temp.path().join("unsafe-plugin");
    std::fs::create_dir_all(source.join("bin")).expect("create directory");
    std::fs::write(source.join("bin").join("tool.sh"), "echo hi\n").expect("write file");
    std::fs::write(
        source.join("plugin.toml"),
        "[plugin]\nname = \"unsafe-plugin\"\nversion = \"0.1.0\"\n\n[permissions]\nnetwork = true\nexecute = true\nfs_write = [\".lode/generated\"]\n",
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "add"])
        .arg(&source)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsafe permission"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "add", "--allow-unsafe"])
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("added plugin unsafe-plugin"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "info", "unsafe-plugin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("network\tok"))
        .stdout(predicate::str::contains("execute\tok"))
        .stdout(predicate::str::contains("fs_write\t.lode/generated"));
}

#[test]
fn plugin_add_rejects_unsafe_fs_write_permission() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let source = temp.path().join("bad-plugin");
    std::fs::create_dir_all(&source).expect("create directory");
    std::fs::write(
        source.join("plugin.toml"),
        "[plugin]\nname = \"bad-plugin\"\n\n[permissions]\nfs_write = [\"../outside\"]\n",
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "add", "--allow-unsafe"])
        .arg(&source)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsafe relative path"));
}

#[test]
fn plugin_add_requires_manifest() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let source = temp.path().join("manifestless-plugin");
    std::fs::create_dir_all(source.join("templates")).expect("create directory");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "add"])
        .arg(&source)
        .assert()
        .failure()
        .stderr(predicate::str::contains("plugin manifest required"));
}

#[test]
fn log_commands_read_and_clear_daemon_log() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .args(["log", "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("log initialised"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "start"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["log", "daemon", "--tail", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon started"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["log", "clear"])
        .assert()
        .success()
        .stdout(predicate::str::contains("logs cleared"));
}

#[test]
fn self_info_clean_upgrade_and_completions_work() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["self", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("global_dir"))
        .stdout(predicate::str::contains("schema_version\t3"))
        .stdout(predicate::str::contains("templates\t"))
        .stdout(predicate::str::contains("upgrade_cache\t"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["self", "clean", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("upgrade"))
        .stdout(predicate::str::contains("daemon-state.json"));

    let root = temp.path().join(".lode");
    std::fs::create_dir_all(root.join("cache").join("upgrade")).expect("create directory");
    std::fs::write(
        root.join("cache").join("upgrade").join("lode-new"),
        "binary\n",
    )
    .expect("write file");
    std::fs::write(root.join("cache").join("daemon-state.txt"), "active\n").expect("write file");
    std::fs::write(root.join("cache").join("daemon-state.json"), "{}\n").expect("write file");
    std::fs::create_dir_all(root.join("logs")).expect("create directory");
    std::fs::write(root.join("logs").join("daemon.log.1"), "old\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .args(["self", "clean"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cleaned"));

    assert!(!root.join("cache").join("upgrade").exists());
    assert!(!root.join("cache").join("daemon-state.txt").exists());
    assert!(!root.join("cache").join("daemon-state.json").exists());
    assert!(!root.join("logs").join("daemon.log.1").exists());

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("is installed"))
        .stdout(predicate::str::contains("staged_upgrade\tnone"));

    let upgrade_dir = root.join("cache").join("upgrade");
    std::fs::create_dir_all(&upgrade_dir).expect("create directory");
    let candidate = upgrade_dir.join("lode-new");
    std::fs::write(&candidate, "binary\n").expect("write file");
    let manifest = upgrade_dir.join("latest.json");
    std::fs::write(
        &manifest,
        serde_json::json!({
            "schema_version": 3,
            "version": "0.2.0",
            "binary": "lode-new",
            "checksum": test_content_hash("binary\n"),
        })
        .to_string(),
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--check", "--manifest"])
        .arg(&manifest)
        .assert()
        .success()
        .stdout(predicate::str::contains("staged_upgrade\t0.2.0"))
        .stdout(predicate::str::contains("verified"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--manifest"])
        .arg(&manifest)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would verify staged upgrade 0.2.0",
        ));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--manifest"])
        .arg(&manifest)
        .assert()
        .success()
        .stdout(predicate::str::contains("upgrade staged\t0.2.0"));

    let upgrade_state =
        std::fs::read_to_string(upgrade_dir.join("upgrade-state.json")).expect("read file");
    assert!(upgrade_state.contains("\"schema_version\": 3"));
    assert!(upgrade_state.contains("\"version\": \"0.2.0\""));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--rollback", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "would rollback staged upgrade 0.2.0",
        ));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--rollback"])
        .assert()
        .success()
        .stdout(predicate::str::contains("upgrade rollback cleared\t0.2.0"));

    assert!(!upgrade_dir.join("upgrade-state.json").exists());

    lode()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -W"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("_lode_chdir_hook"))
        .stdout(predicate::str::contains("lp()"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["completions", "fish", "--install", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would write fish completions"))
        .stdout(predicate::str::contains(
            "would record completion install receipt",
        ))
        .stdout(predicate::str::contains("source"));

    let completion_file = temp.path().join("lode.ps1");
    lode()
        .env("LODE_CONFIG", &config)
        .args(["completions", "powershell", "--install", "--out"])
        .arg(&completion_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote powershell completions"))
        .stdout(predicate::str::contains("add to $PROFILE"))
        .stdout(predicate::str::contains(". \""));

    let completion = std::fs::read_to_string(completion_file).expect("read file");
    assert!(completion.contains("Register-ArgumentCompleter"));
    assert!(completion.contains("Invoke-LodePromptHook"));

    let receipt = std::fs::read_to_string(root.join("completions").join("install-receipt.json"))
        .expect("read file");
    assert!(receipt.contains("\"schema_version\": 3"));
    assert!(receipt.contains("\"shell\": \"powershell\""));
    assert!(receipt.contains("lode.ps1"));
    assert!(receipt.contains("add to $PROFILE"));
}

#[test]
fn task_list_reads_makefile_targets() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("Makefile"),
        "alpha: ## Alpha task\n\t@echo alpha\nbeta:\n\t@echo beta\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["task", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn tauri_and_minecraft_doctor_report_local_files() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(temp.path().join("src-tauri")).expect("create directory");
    std::fs::write(temp.path().join("package.json"), "{}\n").expect("write file");
    std::fs::write(temp.path().join("build.gradle"), "\n").expect("write file");
    std::fs::create_dir_all(temp.path().join("src").join("main")).expect("create directory");

    lode()
        .current_dir(temp.path())
        .args(["tauri", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("src-tauri\tok"));

    lode()
        .current_dir(temp.path())
        .args(["mc", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gradle\tok"));
}

#[test]
fn toolchain_status_detects_project_files() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname='x'\n").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["toolchain", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rust"))
        .stdout(predicate::str::contains("Cargo.toml"));
}

#[test]
fn toolchain_add_use_pin_and_remove_are_file_backed() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["toolchain", "add", "rust", "stable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("toolchain added"));

    lode()
        .current_dir(temp.path())
        .args(["toolchain", "use", "rust", "stable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("toolchain active"));

    assert!(temp.path().join("rust-toolchain.toml").exists());

    lode()
        .current_dir(temp.path())
        .args(["toolchain", "pin", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pinned rust stable"));

    lode()
        .current_dir(temp.path())
        .args(["toolchain", "remove", "rust", "stable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("toolchain removed"));
}

#[test]
fn profile_use_new_and_delete_update_profile_state() {
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
        .success()
        .stdout(predicate::str::contains("active profile"));

    assert!(std::fs::read_to_string(&config)
        .expect("read file")
        .contains("active_profile = \"systems/rust-cli\""));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["profile", "new", "mine"])
        .assert()
        .success();
    assert!(temp
        .path()
        .join(".lode")
        .join("profiles")
        .join("mine.toml")
        .exists());

    lode()
        .env("LODE_CONFIG", &config)
        .args(["profile", "delete", "mine"])
        .assert()
        .success();
    assert!(!temp
        .path()
        .join(".lode")
        .join("profiles")
        .join("mine.toml")
        .exists());
}

#[test]
fn template_reset_and_validate_use_embedded_defaults() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    std::fs::write(
        temp.path()
            .join(".lode")
            .join("templates")
            .join("root")
            .join("README.md"),
        "custom\n",
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .args(["template", "diff", "root/README.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("template differs"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["template", "reset", "root/README.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reset template"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["template", "validate", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("validated"));
}

#[test]
fn time_today_without_log_reports_zero() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["time", "today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("today\t0s"));
}

#[test]
fn time_report_reads_project_log_and_writes_markdown() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let lode_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&lode_dir).expect("create directory");
    std::fs::write(
        lode_dir.join("time-log.json"),
        r#"{
  "sessions": [
    {
      "started_at": "2026-06-10T08:00:00Z",
      "seconds": 3661,
      "project": "demo",
      "task": "implementation"
    }
  ]
}
"#,
    )
    .expect("write file");
    let report = temp.path().join("time-report.md");

    lode()
        .current_dir(temp.path())
        .args(["time", "report", "--out"])
        .arg(&report)
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote time report"));

    let contents = std::fs::read_to_string(report).expect("read file");
    assert!(contents.contains("1h 1m 1s"));
    assert!(contents.contains("implementation"));
}

#[test]
fn time_show_report_and_clear_support_filters() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let lode_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&lode_dir).expect("create directory");
    std::fs::write(
        lode_dir.join("time-log.json"),
        r#"{
  "sessions": [
    {
      "started_at": "2026-05-01T08:00:00Z",
      "seconds": 60,
      "project": "demo",
      "task": "old"
    },
    {
      "started_at": "2026-06-10T08:00:00Z",
      "seconds": 120,
      "project": "demo",
      "task": "new"
    }
  ]
}
"#,
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["time", "show", "--since", "2026-06-01", "--by", "task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("2m 0s"));

    lode()
        .current_dir(temp.path())
        .args([
            "time",
            "report",
            "--since",
            "2026-06-01",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"task\": \"new\""));

    lode()
        .current_dir(temp.path())
        .args(["time", "clear", "--before", "2026-06-01", "--confirm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed 1 session"));
}

#[test]
fn time_clear_confirm_removes_log() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let lode_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&lode_dir).expect("create directory");
    std::fs::write(lode_dir.join("time-log.json"), "{\"sessions\":[]}").expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["time", "clear", "--confirm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("time log cleared"));

    assert!(!lode_dir.join("time-log.json").exists());
}

#[test]
fn file_add_list_check_remove_round_trip() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    std::fs::write(temp.path().join("hello.txt"), "hello world\n").expect("write file");
    std::fs::create_dir_all(temp.path().join("sub")).expect("create dir");
    std::fs::write(temp.path().join("sub").join("nested.txt"), "nested\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "add", "hello.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("added"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "add", "sub/nested.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("added"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.txt"))
        .stdout(predicate::str::contains("nested.txt"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));

    std::fs::write(temp.path().join("hello.txt"), "modified\n").expect("modify file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("modified"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "remove", "hello.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nested.txt"))
        .stdout(predicate::str::contains("hello.txt").not());
}

#[test]
fn file_list_supports_json_output() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    std::fs::write(temp.path().join("foo.txt"), "foo\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "add", "foo.txt"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "list", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\": \"foo.txt\""));
}

#[test]
fn context_compile_respects_token_budget() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    std::fs::create_dir_all(temp.path().join("_ctx_")).expect("create directory");
    std::fs::write(
        temp.path().join("_ctx_").join("001-project.md"),
        "# Project Summary\n\nThis is a test project.\n",
    )
    .expect("write file");
    std::fs::write(
        temp.path().join("_ctx_").join("002-arch.md"),
        "# Architecture\n\nUses layered design.\n",
    )
    .expect("write file");
    std::fs::write(
        temp.path().join("_ctx_").join("003-long.md"),
        "A very long context file that repeats itself. ".repeat(100),
    )
    .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["context", "compile", "--budget", "500"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Context Compiled"));

    let compiled = temp
        .path()
        .join(".lode")
        .join("context")
        .join("COMPILED.md");
    assert!(compiled.exists());
    let contents = std::fs::read_to_string(&compiled).expect("read file");
    assert!(contents.contains("Project Summary"));
    assert!(contents.contains("Architecture"));
    assert!(contents.contains("## Full Content"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["context", "compile", "--budget", "500", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_estimated_tokens\""))
        .stdout(predicate::str::contains("\"total_files\""));
}

#[test]
fn context_compile_without_context_files_reports_empty() {
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
        .args(["context", "compile"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 files included"));
}

#[test]
fn agent_policy_generates_all_expected_files() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let project_dir = temp.path().join("my-project");
    std::fs::create_dir_all(&project_dir).expect("create directory");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project_dir)
        .args(["agent", "policy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent Policy Generated"))
        .stdout(predicate::str::contains("AGENTS.md"))
        .stdout(predicate::str::contains("CLAUDE.md"))
        .stdout(predicate::str::contains(".cursorrules"))
        .stdout(predicate::str::contains("9 files written"));

    assert!(project_dir.join("AGENTS.md").exists());
    assert!(project_dir.join("CLAUDE.md").exists());
    assert!(project_dir.join("CODEX.md").exists());
    assert!(project_dir.join(".cursorrules").exists());
    assert!(project_dir.join(".windsurfrules").exists());
    assert!(project_dir.join(".mcp.json").exists());
    assert!(project_dir
        .join(".lode")
        .join("context")
        .join("PLAN.md")
        .exists());
    assert!(project_dir
        .join(".lode")
        .join("context")
        .join("CONSTRAINTS.md")
        .exists());
    assert!(project_dir
        .join(".lode")
        .join("context")
        .join("TASKS.md")
        .exists());

    let agents_content = std::fs::read_to_string(project_dir.join("AGENTS.md")).expect("read file");
    assert!(agents_content.contains("my-project"));
    assert!(agents_content.contains("## Core Principles"));
    assert!(agents_content.contains("## Contract"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project_dir)
        .args(["agent", "policy", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files_written\""))
        .stdout(predicate::str::contains("\"AGENTS.md\""));
}

#[test]
fn agent_policy_files_are_registered_in_manifest() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let project_dir = temp.path().join("policy-project");
    std::fs::create_dir_all(&project_dir).expect("create directory");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project_dir)
        .args(["agent", "policy"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project_dir)
        .args(["file", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AGENTS.md"))
        .stdout(predicate::str::contains("CLAUDE.md"))
        .stdout(predicate::str::contains("agent"))
        .stdout(predicate::str::contains("Managed By"));
}

#[test]
fn verify_changed_detects_modified_files() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    std::fs::write(temp.path().join("stable.txt"), "content\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "add", "stable.txt"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["verify", "--changed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unchanged"));

    std::fs::write(temp.path().join("stable.txt"), "tampered\n").expect("modify file");

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["verify", "--changed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MODIFIED"));
}

#[test]
fn verify_changed_json_output() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    std::fs::write(temp.path().join("data.txt"), "data\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["file", "add", "data.txt"])
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["verify", "--changed", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_files\""))
        .stdout(predicate::str::contains("\"ok\":"));
}

#[test]
fn verify_changed_without_manifest_reports_gracefully() {
    let temp = tempfile::tempdir().expect("create temp dir");

    lode()
        .current_dir(temp.path())
        .args(["verify", "--changed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No file manifest found"));
}

#[test]
fn template_bundle_capture_show_validate_verify() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);

    // Create a source directory with some files to capture
    let source = temp.path().join("mysrc");
    std::fs::create_dir_all(&source).expect("create source");
    std::fs::write(source.join("README.md"), "# Hello {{ project }}\n").expect("write");
    std::fs::write(source.join("main.rs"), "fn main() {}\n").expect("write");
    std::fs::create_dir_all(source.join("assets")).expect("create assets dir");
    std::fs::write(source.join("assets").join("logo.png"), "fake-png").expect("write");

    let bundle_dir = temp.path().join("mybundle");

    // Capture
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args([
            "template-bundle",
            "capture",
            source.to_str().unwrap(),
            bundle_dir.to_str().unwrap(),
            "--name",
            "mybundle",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("captured"))
        .stdout(predicate::str::contains("inline files"));

    assert!(bundle_dir.join("mybundle.toml").exists(), "manifest exists");
    assert!(bundle_dir.join("assets").exists(), "assets dir exists");

    // Show
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["template-bundle", "show", bundle_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_version"))
        .stdout(predicate::str::contains("mybundle"));

    // Validate
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["template-bundle", "validate", bundle_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));

    // Verify (may report missing assets if asset source paths are nested)
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["template-bundle", "verify", bundle_dir.to_str().unwrap()])
        .assert()
        .success();

    // Preview
    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path())
        .args(["template-bundle", "preview", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("source:"))
        .stdout(predicate::str::contains("inline files:"));
}
