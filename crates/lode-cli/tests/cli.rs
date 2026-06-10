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
fn git_branch_formats_slug() {
    lode()
        .args(["git", "branch", "feat", "User Login Flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feat/user-login-flow"));
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
        .args(["workspace", "graph"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-> crates/app"));
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
        .args(["serve", "--no-color", "--no-live"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lode serve"))
        .stdout(predicate::str::contains("PROJECT HEALTH"))
        .stdout(predicate::str::contains("CROSS-PROJECT REGISTRY"));
}
