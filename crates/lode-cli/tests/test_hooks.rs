#[path = "common/mod.rs"]
mod common;

use common::*;
use predicates::prelude::*;

#[test]
fn hooks_list_status_and_test_are_available() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(temp.path().join(".git").join("hooks")).expect("create directory");

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
fn hooks_run_executes_project_hook() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let hooks = temp.path().join(".lode").join("hooks");
    std::fs::create_dir_all(&hooks).expect("create directory");
    std::fs::write(
        hooks.join("post-init.ps1"),
        "Set-Content -NoNewline -Path hook-output.txt -Value ran\n",
    )
    .expect("write file");

    lode()
        .current_dir(temp.path())
        .args(["hooks", "run", "post-init", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would run hook project"))
        .stdout(predicate::str::contains("post-init.ps1"));

    lode()
        .current_dir(temp.path())
        .args(["hooks", "run", "post-init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("running hook project"));

    assert_eq!(
        std::fs::read_to_string(temp.path().join("hook-output.txt")).expect("read file"),
        "ran"
    );
}

#[test]
fn hooks_discover_plugin_global_and_project_sources() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let lode_root = temp.path().join(".lode");
    let plugin_source = temp.path().join("audit-pack");
    let plugin_hooks = plugin_source.join("hooks");
    let global_hooks = lode_root.join("hooks");
    let project = temp.path().join("project");
    let project_hooks = project.join(".lode").join("hooks");
    std::fs::create_dir_all(&plugin_hooks).expect("create directory");
    std::fs::create_dir_all(&global_hooks).expect("create directory");
    std::fs::create_dir_all(&project_hooks).expect("create directory");
    std::fs::write(
        plugin_source.join("plugin.toml"),
        "[plugin]\nname = \"audit-pack\"\nversion = \"0.1.0\"\n\n[permissions]\nexecute = true\n",
    )
    .expect("write file");
    std::fs::write(plugin_hooks.join("post-init.sh"), "echo plugin\n").expect("write file");
    std::fs::write(global_hooks.join("post-init.py"), "print('global')\n").expect("write file");
    std::fs::write(project_hooks.join("post-init.ps1"), "Write-Host project\n")
        .expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .arg("setup")
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .args(["plugin", "add", "--allow-unsafe"])
        .arg(&plugin_source)
        .assert()
        .success();

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
fn hooks_run_passes_plugin_permission_environment() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let plugin_source = temp.path().join("env-pack");
    let plugin_hooks = plugin_source.join("hooks");
    let project = temp.path().join("project");
    std::fs::create_dir_all(&plugin_hooks).expect("create directory");
    std::fs::create_dir_all(&project).expect("create directory");
    std::fs::write(
        plugin_source.join("plugin.toml"),
        "[plugin]\nname = \"env-pack\"\nversion = \"0.1.0\"\n\n[permissions]\nexecute = true\nnetwork = true\nfs_write = [\"hook-output.txt\"]\n",
    )
    .expect("write file");
    std::fs::write(
        plugin_hooks.join("post-init.ps1"),
        "$items = @($env:LODE_HOOK_EVENT, $env:LODE_PLUGIN_NAME, $env:LODE_PLUGIN_ALLOW_NETWORK, $env:LODE_PLUGIN_ALLOW_EXECUTE, $env:LODE_PLUGIN_FS_WRITE); Set-Content -NoNewline -Path hook-output.txt -Value ($items -join '|')\n",
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
        .arg(&plugin_source)
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["hooks", "run", "post-init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("running hook plugin:env-pack"));

    assert_eq!(
        std::fs::read_to_string(project.join("hook-output.txt")).expect("read file"),
        "post-init|env-pack|true|true|hook-output.txt"
    );
}

#[test]
fn hooks_reject_plugin_writes_outside_declared_paths() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let plugin_source = temp.path().join("write-pack");
    let plugin_hooks = plugin_source.join("hooks");
    let project = temp.path().join("project");
    std::fs::create_dir_all(&plugin_hooks).expect("create directory");
    std::fs::create_dir_all(&project).expect("create directory");
    std::fs::write(
        plugin_source.join("plugin.toml"),
        "[plugin]\nname = \"write-pack\"\nversion = \"0.1.0\"\n\n[permissions]\nexecute = true\nfs_write = [\"allowed.txt\"]\n",
    )
    .expect("write file");
    std::fs::write(
        plugin_hooks.join("post-init.ps1"),
        "Set-Content -NoNewline -Path blocked.txt -Value nope\n",
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
        .arg(&plugin_source)
        .assert()
        .success();

    lode()
        .env("LODE_CONFIG", &config)
        .current_dir(&project)
        .args(["hooks", "run", "post-init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "wrote outside declared fs_write paths: blocked.txt",
        ));
}

#[test]
fn hooks_reject_plugin_hooks_without_execute_permission() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let config = isolated_config(&temp);
    let plugin_hooks = temp
        .path()
        .join(".lode")
        .join("plugins")
        .join("audit-pack")
        .join("hooks");
    std::fs::create_dir_all(&plugin_hooks).expect("create directory");
    std::fs::write(plugin_hooks.join("post-init.sh"), "echo plugin\n").expect("write file");

    lode()
        .env("LODE_CONFIG", &config)
        .args(["hooks", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "does not declare permissions.execute = true",
        ));
}
