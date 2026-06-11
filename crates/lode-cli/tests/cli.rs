use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn lode() -> Command {
    Command::cargo_bin("lode").unwrap()
}

fn isolated_config(temp: &TempDir) -> String {
    temp.path()
        .join(".lode")
        .join("config.toml")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn setup_defaults_creates_config_and_dirs() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .arg("--defaults")
        .assert()
        .success()
        .stdout(predicate::str::contains("lode initialised"));

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
    .unwrap()
    .contains("Mozilla Public License"));
}

#[test]
fn setup_and_export_respect_template_dir_override() {
    let temp = tempfile::tempdir().unwrap();
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
        .unwrap()
        .contains("templates/root/README.md"));
}

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
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
    let raw = std::fs::read_to_string(&project_config).unwrap();
    std::fs::write(
        &project_config,
        raw.replacen("schema_version = 3", "schema_version = 2", 1),
    )
    .unwrap();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(temp.path().join("project-app"))
        .args(["config", "validate", "--project"])
        .assert()
        .code(6)
        .stderr(predicate::str::contains("schema version mismatch"));
}

#[test]
fn template_and_snippet_lists_support_json() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["template", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("root/README.md"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["snippet", "list", "--format", "json", "--lang", "rs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"serde-struct\""));
}

#[test]
fn config_set_updates_global_config() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
        .unwrap()
        .contains("author = \"Your Name\""));
}

#[test]
fn init_dry_run_writes_nothing() {
    let temp = tempfile::tempdir().unwrap();
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
fn init_creates_default_project_layout() {
    let temp = tempfile::tempdir().unwrap();
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
        .unwrap()
        .contains(".env"));
    assert!(project.join(".git").exists());
}

#[test]
fn init_with_rust_profile_creates_language_files() {
    let temp = tempfile::tempdir().unwrap();
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
            .unwrap()
            .contains("lode.startDaemonOnOpen")
    );
    assert!(
        std::fs::read_to_string(project.join(".vscode").join("tasks.json"))
            .unwrap()
            .contains("lode: open dashboard")
    );
}

#[test]
fn add_editor_integrations_scaffolds_files() {
    let temp = tempfile::tempdir().unwrap();
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

    assert!(
        std::fs::read_to_string(project.join(".zed").join("tasks.json"))
            .unwrap()
            .contains("lode: daemon start")
    );
    assert!(std::fs::read_to_string(
        project
            .join(".config")
            .join("nvim")
            .join("lua")
            .join("lode.lua")
    )
    .unwrap()
    .contains("daemon_auto_start"));
}

#[test]
fn rerunning_init_returns_exists_code() {
    let temp = tempfile::tempdir().unwrap();
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
fn check_reports_convention_violations_with_exit_code_2() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("BadName.rs"), "").unwrap();

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
        .stdout(predicate::str::contains("BadName.rs -> bad_name.rs"));
}

#[test]
fn check_fix_renames_convention_violations() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("BadName.rs"), "").unwrap();

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
fn rules_list_and_validate_use_config() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let file = temp.path().join("main.rs");
    std::fs::write(&file, "fn main() {}\n").unwrap();

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

    let contents = std::fs::read_to_string(file).unwrap();
    assert!(contents.contains("Generated with Lode"));
    assert!(contents.contains("MIT OR Apache-2.0"));
}

#[test]
fn scan_secrets_returns_exit_code_7() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join(".env"), "API_KEY=real-value\n").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["scan", "secrets"])
        .assert()
        .code(7)
        .stdout(predicate::str::contains("suspicious credential assignment"));
}

#[test]
fn scan_secrets_quiet_supports_staged_flag() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join(".env"), "API_KEY=real-value\n").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["scan", "secrets", "--quiet", "--staged"])
        .assert()
        .code(7)
        .stdout(predicate::str::contains(
            "scanning staged-compatible project path",
        ));
}

#[test]
fn init_registers_project_and_projects_list_shows_it() {
    let temp = tempfile::tempdir().unwrap();
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
fn projects_prune_removes_missing_projects() {
    let temp = tempfile::tempdir().unwrap();
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

    std::fs::remove_dir_all(temp.path().join("gone-app")).unwrap();

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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let project = temp.path().join("manual-app");
    std::fs::create_dir_all(&project).unwrap();

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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let project = temp.path().join("json-app");
    std::fs::create_dir_all(&project).unwrap();

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
            "--format",
            "json",
            "--sort",
            "last-seen",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"json-app\""));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["license", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MIT.txt"));
}

#[test]
fn projects_health_supports_json_and_refresh() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let project = temp.path().join("healthy-app");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("README.md"), "# App\n").unwrap();
    std::fs::write(project.join("LICENSE"), "MIT\n").unwrap();
    std::fs::write(project.join(".env.example"), "APP_NAME=app\n").unwrap();

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
        .args(["projects", "health", "--json", "--refresh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"healthy-app\""))
        .stdout(predicate::str::contains("\"score\""));
}

#[test]
fn env_sync_and_check_use_env_example() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join(".env.example"),
        "APP_NAME=test\nLOG_LEVEL=debug\n",
    )
    .unwrap();

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
    let temp = tempfile::tempdir().unwrap();

    lode()
        .current_dir(temp.path())
        .args(["env", "add", "API_URL"])
        .assert()
        .success();

    assert!(std::fs::read_to_string(temp.path().join(".env.example"))
        .unwrap()
        .contains("API_URL="));
}

#[test]
fn env_add_supports_default_comment_and_secret() {
    let temp = tempfile::tempdir().unwrap();

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

    let example = std::fs::read_to_string(temp.path().join(".env.example")).unwrap();
    let env = std::fs::read_to_string(temp.path().join(".env")).unwrap();
    assert!(example.contains("# Local API token"));
    assert!(example.contains("API_TOKEN=\n"));
    assert!(env.contains("API_TOKEN=dev-token"));
}

#[test]
fn license_set_and_check_write_license_file() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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

    let exported = std::fs::read_to_string(out).unwrap();
    assert!(exported.contains("rs:serde-struct"));
    assert!(serde_json::from_str::<serde_json::Value>(&exported).is_ok());
}

#[test]
fn snippet_export_accepts_zed_format() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let target = temp.path().join("notes.txt");
    std::fs::write(&target, "before\nafter\n").unwrap();

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

    let contents = std::fs::read_to_string(&target).unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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

    assert!(temp.path().join("docs").join("docker-basic.md").exists());
}

#[test]
fn recipe_new_and_compose_are_file_backed() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
        .stdout(predicate::str::contains("step 1 [make] verify"));
}

#[test]
fn commands_run_executes_project_local_shell_macro() {
    let temp = tempfile::tempdir().unwrap();
    let commands_dir = temp.path().join(".lode").join("commands");
    std::fs::create_dir_all(&commands_dir).unwrap();
    std::fs::write(
        commands_dir.join("touch-file.toml"),
        "slug = \"touch-file\"\n\n[[steps]]\nkind = \"shell\"\nrun = \"echo made> macro-output.txt\"\n",
    )
    .unwrap();

    lode()
        .current_dir(temp.path())
        .args(["commands", "run", "touch-file"])
        .assert()
        .success()
        .stdout(predicate::str::contains("step 1 [shell]"));

    assert!(temp.path().join("macro-output.txt").exists());
}

#[test]
fn custom_command_direct_invocation_supports_help_and_dry_run() {
    let temp = tempfile::tempdir().unwrap();
    let commands_dir = temp.path().join(".lode").join("commands");
    std::fs::create_dir_all(&commands_dir).unwrap();
    std::fs::write(
        commands_dir.join("deploy.toml"),
        "description = \"Deploy app\"\n\n[[steps]]\nkind = \"shell\"\nrun = \"echo deployed> deployed.txt\"\n",
    )
    .unwrap();

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
        .stdout(predicate::str::contains("step 1 [shell]"))
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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let exported = temp.path().join("commands.json");

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
        .arg(&exported)
        .assert()
        .success()
        .stdout(predicate::str::contains("exported"));

    assert!(std::fs::read_to_string(&exported)
        .unwrap()
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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let source = temp.path().join("my-plugin");
    std::fs::create_dir_all(source.join("templates")).unwrap();
    std::fs::write(source.join("templates").join("README.md"), "# Plugin\n").unwrap();
    std::fs::write(
        source.join("plugin.toml"),
        "[plugin]\nname = \"my-plugin\"\nversion = \"1.2.3\"\ndescription = \"Test plugin for local templates\"\n",
    )
    .unwrap();

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
        .stdout(predicate::str::contains("templates\tok"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "search", "templates", "--format", "json"])
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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let source = temp.path().join("unsafe-plugin");
    std::fs::create_dir_all(source.join("bin")).unwrap();
    std::fs::write(source.join("bin").join("tool.sh"), "echo hi\n").unwrap();
    std::fs::write(
        source.join("plugin.toml"),
        "[plugin]\nname = \"unsafe-plugin\"\nversion = \"0.1.0\"\n\n[permissions]\nnetwork = true\nexecute = true\nfs_write = [\".lode/generated\"]\n",
    )
    .unwrap();

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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let source = temp.path().join("bad-plugin");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        source.join("plugin.toml"),
        "[plugin]\nname = \"bad-plugin\"\n\n[permissions]\nfs_write = [\"../outside\"]\n",
    )
    .unwrap();

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
fn mcp_lists_tools_resources_and_prompts() {
    lode()
        .args(["mcp", "--list-tools", "--list-resources", "--list-prompts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"tools\""))
        .stdout(predicate::str::contains("lode_config_show"))
        .stdout(predicate::str::contains("lode://config"))
        .stdout(predicate::str::contains("lode-project-review"));
}

#[test]
fn mcp_stdio_handles_tool_calls() {
    lode()
        .write_stdin(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lode_template_list","arguments":{}}}
"#,
        )
        .arg("mcp")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"serverInfo\""))
        .stdout(predicate::str::contains("\"tools\""))
        .stdout(predicate::str::contains("root/README.md"));
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

#[test]
fn agent_sync_plan_and_export_are_file_backed() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("_ref_")).unwrap();
    std::fs::create_dir_all(temp.path().join("_ctx_")).unwrap();
    std::fs::write(
        temp.path().join("_ref_").join("ARCHITECTURE.md"),
        "# Arch\n",
    )
    .unwrap();
    std::fs::write(temp.path().join("AGENTS.md"), "# Agent\n").unwrap();

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
fn git_branch_formats_slug() {
    lode()
        .args(["git", "branch", "feat", "User Login Flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feat/user-login-flow"));
}

#[test]
fn git_setup_commands_write_metadata_files() {
    let temp = tempfile::tempdir().unwrap();

    lode()
        .current_dir(temp.path())
        .args(["git", "sign-setup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("git signing setup recorded"));

    lode()
        .current_dir(temp.path())
        .args([
            "git",
            "remote-setup",
            "--provider",
            "github",
            "--visibility",
            "public",
            "--token-env",
            "GH_TOKEN",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("git remote setup recorded"));

    assert!(temp.path().join(".lode").join("git-signing.toml").exists());
    assert!(
        std::fs::read_to_string(temp.path().join(".lode").join("remote.toml"))
            .unwrap()
            .contains("GH_TOKEN")
    );
}

#[test]
fn git_changelog_can_write_plain_output() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("CHANGELOG.generated");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    std::fs::write(temp.path().join("file.txt"), "hello\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "feat: hello"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    lode()
        .current_dir(temp.path())
        .args(["git", "changelog", "--format", "plain", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote changelog"));

    assert!(std::fs::read_to_string(out)
        .unwrap()
        .contains("feat: hello"));
}

#[test]
fn git_hooks_install_status_and_uninstall() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join(".git").join("hooks")).unwrap();

    lode()
        .current_dir(temp.path())
        .args(["git", "install-hooks"])
        .assert()
        .success()
        .stdout(predicate::str::contains("installed"));

    lode()
        .current_dir(temp.path())
        .args(["git", "hooks-status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-commit\tmanaged"))
        .stdout(predicate::str::contains("pre-push\tmanaged"));

    lode()
        .current_dir(temp.path())
        .args(["git", "uninstall-hooks"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    assert!(!temp
        .path()
        .join(".git")
        .join("hooks")
        .join("pre-commit")
        .exists());
}

#[test]
fn hooks_list_status_and_test_are_available() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join(".git").join("hooks")).unwrap();

    lode()
        .current_dir(temp.path())
        .args(["hooks", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-commit").not());

    lode()
        .current_dir(temp.path())
        .args(["hooks", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-push"));

    lode()
        .current_dir(temp.path())
        .args(["hooks", "test", "pre-commit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lode scan secrets"));
}

#[test]
fn hooks_discover_plugin_global_and_project_sources() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let lode_root = temp.path().join(".lode");
    let plugin_hooks = lode_root.join("plugins").join("audit-pack").join("hooks");
    let global_hooks = lode_root.join("hooks");
    let project = temp.path().join("project");
    let project_hooks = project.join(".lode").join("hooks");
    std::fs::create_dir_all(&plugin_hooks).unwrap();
    std::fs::create_dir_all(&global_hooks).unwrap();
    std::fs::create_dir_all(&project_hooks).unwrap();
    std::fs::write(
        lode_root
            .join("plugins")
            .join("audit-pack")
            .join("plugin.toml"),
        "[plugin]\nname = \"audit-pack\"\nversion = \"0.1.0\"\n\n[permissions]\nexecute = true\n",
    )
    .unwrap();
    std::fs::write(plugin_hooks.join("post-init.sh"), "echo plugin\n").unwrap();
    std::fs::write(global_hooks.join("post-init.py"), "print('global')\n").unwrap();
    std::fs::write(project_hooks.join("post-init.ps1"), "Write-Host project\n").unwrap();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["hooks", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("post-init\tplugin:audit-pack"))
        .stdout(predicate::str::contains("post-init\tglobal"))
        .stdout(predicate::str::contains("post-init\tproject"));

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["hooks", "test", "post-init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("plugin:audit-pack\tsh"))
        .stdout(predicate::str::contains("global\tpython"))
        .stdout(predicate::str::contains("project\tpowershell"));
}

#[test]
fn hooks_reject_plugin_hooks_without_execute_permission() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    let plugin_hooks = temp
        .path()
        .join(".lode")
        .join("plugins")
        .join("audit-pack")
        .join("hooks");
    std::fs::create_dir_all(&plugin_hooks).unwrap();
    std::fs::write(plugin_hooks.join("post-init.sh"), "echo plugin\n").unwrap();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["hooks", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "does not declare permissions.execute = true",
        ));
}

#[test]
fn export_import_round_trips_lodepack() {
    let source = tempfile::tempdir().unwrap();
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

    let dest = tempfile::tempdir().unwrap();
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
            .unwrap()
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
    let source = tempfile::tempdir().unwrap();
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
    .unwrap();
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

    let raw = std::fs::read_to_string(&pack).unwrap();
    assert!(!raw.contains("templates/root/README.md"));
    assert!(!raw.contains("snippets/rs/serde-struct.snippet"));
    assert!(!raw.contains("commands/health.toml"));
    assert!(raw.contains("registry.json"));

    let dest = tempfile::tempdir().unwrap();
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
fn sync_refreshes_agent_context_and_supports_dry_run() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    std::fs::create_dir_all(temp.path().join("_ref_")).unwrap();
    std::fs::create_dir_all(temp.path().join("_ctx_")).unwrap();
    std::fs::write(temp.path().join("_ctx_").join("NOTES.md"), "# Notes\n").unwrap();

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
    let temp = tempfile::tempdir().unwrap();
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
    std::fs::remove_file(project.join("README.md")).unwrap();

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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    std::fs::write(temp.path().join("README.md"), "# App\n").unwrap();
    std::fs::write(temp.path().join("LICENSE"), "MIT\n").unwrap();
    std::fs::write(temp.path().join(".env.example"), "APP_NAME=app\n").unwrap();

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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);
    std::fs::write(temp.path().join("README.md"), "# App\n").unwrap();
    std::fs::write(temp.path().join("LICENSE"), "MIT\n").unwrap();
    std::fs::write(temp.path().join(".env.example"), "APP_NAME=app\n").unwrap();

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
fn workspace_init_add_list_and_graph_are_file_backed() {
    let temp = tempfile::tempdir().unwrap();

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
        .args(["workspace", "remove", "crates/app", "--confirm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace member removed"));
}

#[test]
fn profile_use_new_and_delete_update_profile_state() {
    let temp = tempfile::tempdir().unwrap();
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
        .unwrap()
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
fn init_uses_active_profile_when_profile_flag_is_absent() {
    let temp = tempfile::tempdir().unwrap();
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
fn daemon_start_status_log_and_stop_are_stateful() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["daemon", "start", "--no-rename", "--foreground"])
        .assert()
        .success()
        .stdout(predicate::str::contains("foreground daemon watching"));

    lode()
        .env("LODE_CONFIG", &config)
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
    .unwrap();
    assert!(state.contains("\"events\""));
    assert!(state.contains("\"foreground\""));

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
fn log_commands_read_and_clear_daemon_log() {
    let temp = tempfile::tempdir().unwrap();
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
    let temp = tempfile::tempdir().unwrap();
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
        .stdout(predicate::str::contains("global_dir"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["self", "clean", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would clean"));

    lode()
        .env("LODE_CONFIG", &config)
        .args(["upgrade", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("is installed"));

    lode()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -W"))
        .stdout(predicate::str::contains("_lode_chdir_hook"))
        .stdout(predicate::str::contains("lp()"));

    let completion_file = temp.path().join("lode.ps1");
    lode()
        .env("LODE_CONFIG", &config)
        .args(["completions", "powershell", "--install", "--out"])
        .arg(&completion_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote powershell completions"));

    let completion = std::fs::read_to_string(completion_file).unwrap();
    assert!(completion.contains("Register-ArgumentCompleter"));
    assert!(completion.contains("Invoke-LodePromptHook"));
}

#[test]
fn task_list_reads_makefile_targets() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("Makefile"),
        "alpha: ## Alpha task\n\t@echo alpha\nbeta:\n\t@echo beta\n",
    )
    .unwrap();

    lode()
        .current_dir(temp.path())
        .args(["task", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn gha_add_and_validate_workflows() {
    let temp = tempfile::tempdir().unwrap();

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

#[test]
fn tauri_and_minecraft_doctor_report_local_files() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("src-tauri")).unwrap();
    std::fs::write(temp.path().join("package.json"), "{}\n").unwrap();
    std::fs::write(temp.path().join("build.gradle"), "\n").unwrap();
    std::fs::create_dir_all(temp.path().join("src").join("main")).unwrap();

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
fn cp_new_creates_problem_file() {
    let temp = tempfile::tempdir().unwrap();

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

#[test]
fn toolchain_status_detects_project_files() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname='x'\n").unwrap();

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
    let temp = tempfile::tempdir().unwrap();

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
fn pkg_list_detects_package_manager() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("package.json"), "{}\n").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["pkg", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("manager: npm"))
        .stdout(predicate::str::contains("package.json"));
}

#[test]
fn pkg_update_dry_run_prints_manager_command() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("package.json"), "{}\n").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["pkg", "update", "left-pad", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: npm update left-pad"));
}

#[test]
fn pkg_dry_run_translates_native_commands() {
    let node = tempfile::tempdir().unwrap();
    std::fs::write(node.path().join("pnpm-lock.yaml"), "lockfileVersion: '9'\n").unwrap();

    lode()
        .current_dir(node.path())
        .args(["pkg", "outdated", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: pnpm outdated"));

    lode()
        .current_dir(node.path())
        .args(["pkg", "why", "react", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: pnpm why react"));

    let python = tempfile::tempdir().unwrap();
    std::fs::write(python.path().join("requirements.txt"), "requests==2.0.0\n").unwrap();

    lode()
        .current_dir(python.path())
        .args(["pkg", "info", "requests", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: pip show requests"));

    let go = tempfile::tempdir().unwrap();
    std::fs::write(go.path().join("go.sum"), "example.com/mod v1.0.0 h1:abc\n").unwrap();

    lode()
        .current_dir(go.path())
        .args(["pkg", "audit", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run: go vulncheck ./..."))
        .stdout(predicate::str::contains("would run: lode scan secrets"));
}

#[test]
fn pkg_graph_json_reports_manifest() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname='x'\n").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["pkg", "graph", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\": \"cargo\""));
}

#[test]
fn pkg_graph_dot_reports_manifest() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("package.json"), "{}\n").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["pkg", "graph", "--format", "dot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph packages"))
        .stdout(predicate::str::contains("project -> node"));
}

#[test]
fn release_bumps_cargo_version() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    lode()
        .current_dir(temp.path())
        .args(["release", "--bump", "patch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.1"));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .unwrap()
        .contains("version = \"0.1.1\""));
    assert!(!temp
        .path()
        .join(".lode")
        .join("release.rollback.json")
        .exists());
}

#[test]
fn release_bumps_workspace_cargo_version() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"1.2.3\"\n",
    )
    .unwrap();

    lode()
        .current_dir(temp.path())
        .args(["release", "--bump", "minor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.3.0"));

    assert!(std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .unwrap()
        .contains("version = \"1.3.0\""));
}

#[test]
fn release_dry_run_does_not_write() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("pyproject.toml"),
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    lode()
        .current_dir(temp.path())
        .args(["release", "--bump", "minor", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would update"));

    assert!(std::fs::read_to_string(temp.path().join("pyproject.toml"))
        .unwrap()
        .contains("version = \"0.1.0\""));
}

#[test]
fn serve_renders_dashboard_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let config = isolated_config(&temp);

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();
    std::fs::write(temp.path().join("README.md"), "# App\n").unwrap();
    std::fs::write(temp.path().join("LICENSE"), "MIT\n").unwrap();
    std::fs::write(temp.path().join(".env.example"), "APP_NAME=app\n").unwrap();

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

#[test]
fn template_reset_and_validate_use_embedded_defaults() {
    let temp = tempfile::tempdir().unwrap();
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
    .unwrap();

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
    let temp = tempfile::tempdir().unwrap();

    lode()
        .current_dir(temp.path())
        .args(["time", "today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("today\t0s"));
}

#[test]
fn time_report_reads_project_log_and_writes_markdown() {
    let temp = tempfile::tempdir().unwrap();
    let lode_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&lode_dir).unwrap();
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
    .unwrap();
    let report = temp.path().join("time-report.md");

    lode()
        .current_dir(temp.path())
        .args(["time", "report", "--out"])
        .arg(&report)
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote time report"));

    let contents = std::fs::read_to_string(report).unwrap();
    assert!(contents.contains("1h 1m 1s"));
    assert!(contents.contains("implementation"));
}

#[test]
fn time_show_report_and_clear_support_filters() {
    let temp = tempfile::tempdir().unwrap();
    let lode_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&lode_dir).unwrap();
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
    .unwrap();

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
    let temp = tempfile::tempdir().unwrap();
    let lode_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&lode_dir).unwrap();
    std::fs::write(lode_dir.join("time-log.json"), "{\"sessions\":[]}").unwrap();

    lode()
        .current_dir(temp.path())
        .args(["time", "clear", "--confirm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("time log cleared"));

    assert!(!lode_dir.join("time-log.json").exists());
}
