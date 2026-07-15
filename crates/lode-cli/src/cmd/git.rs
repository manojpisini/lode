#![deny(unsafe_code)]

use lode_core::{LodeError, Process};

use crate::GitCommand;

pub(crate) fn git(command: GitCommand) -> lode_core::Result<()> {
    match command {
        GitCommand::Branch { kind, description } => {
            let branch = format!("{kind}/{description}");
            let status = Process::new("git")?
                .args(["checkout", "-b", &branch])
                .status()?;
            if status.success() {
                println!("created branch {branch}");
            } else {
                return Err(LodeError::Message("failed to create branch".to_string()));
            }
        }
        GitCommand::Commit {
            message,
            r#type,
            scope,
            breaking,
            no_confirm,
        } => {
            let msg = message.unwrap_or_else(|| {
                if no_confirm {
                    String::new()
                } else {
                    crate::cmd::output::input("commit message").unwrap_or_default()
                }
            });
            let scope_part = scope.as_ref().map(|s| format!("({s})")).unwrap_or_default();
            let breaking_mark = if breaking { "!" } else { "" };
            let type_str = r#type.as_deref().unwrap_or("chore");
            let full_msg = format!("{type_str}{scope_part}{breaking_mark}: {msg}");
            let status = Process::new("git")?
                .args(["commit", "-m", &full_msg])
                .status()?;
            if status.success() {
                println!("committed: {full_msg}");
            }
        }
        GitCommand::Tag {
            version,
            no_changelog,
            push,
            message,
        } => {
            let msg = message.unwrap_or_else(|| format!("v{version}"));
            let status = Process::new("git")?
                .args(["tag", "-a", &version, "-m", &msg])
                .status()?;
            if status.success() {
                println!("tagged {version}");
                if push {
                    Process::new("git")?
                        .args(["push", "origin", &version])
                        .status()?;
                    println!("pushed tag {version}");
                }
            }
            if !no_changelog {
                println!("changelog: would update CHANGELOG.md for {version}");
            }
        }
        GitCommand::Changelog { since, out, format } => {
            let since_arg = since.as_deref().unwrap_or("HEAD");
            let output = Process::new("git")?
                .args(["log", since_arg, "--oneline"])
                .output()?;
            let entries = String::from_utf8_lossy(&output.stdout);
            if format == "json" {
                let items: Vec<&str> = entries.lines().collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items)
                        .map_err(|e| LodeError::Message(e.to_string()))?
                );
            } else {
                let path = out.unwrap_or_else(|| "CHANGELOG.md".into());
                let content = format!("# Changelog\n\n## {since_arg}\n\n{entries}");
                std::fs::write(path.as_str(), &content).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?;
                println!("wrote changelog to {path}");
            }
        }
        GitCommand::InstallHooks => {
            let hooks_dir = camino::Utf8PathBuf::from(".git/hooks");
            if !hooks_dir.exists() {
                return Err(LodeError::Message("not a git repository".to_string()));
            }
            for name in ["pre-commit", "pre-push", "pre-receive", "post-commit"] {
                let hook_path = hooks_dir.join(name);
                if hook_path.exists() {
                    println!("hook already exists: {name}");
                    continue;
                }
                let hook_path = hook_path.as_std_path();
                let content =
                    format!("#!/bin/sh\n# lode-managed hook: {name}\nexec lode hooks run {name}\n");
                std::fs::write(hook_path, &content).map_err(|source| LodeError::Io {
                    path: hook_path.display().to_string().into(),
                    source,
                })?;
                println!("installed hook: {name}");
            }
        }
        GitCommand::UninstallHooks => {
            let hooks_dir = std::path::Path::new(".git/hooks");
            for name in ["pre-commit", "pre-push", "pre-receive", "post-commit"] {
                let hook_path = hooks_dir.join(name);
                if hook_path.exists() {
                    std::fs::remove_file(&hook_path).map_err(|source| LodeError::Io {
                        path: hook_path.display().to_string().into(),
                        source,
                    })?;
                    println!("removed hook: {name}");
                }
            }
        }
        GitCommand::HooksStatus => {
            let active = std::path::Path::new(".git/hooks")
                .read_dir()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| !e.file_name().to_string_lossy().ends_with(".sample"))
                        .count()
                })
                .unwrap_or(0);
            println!("active hooks: {active}");
        }
        GitCommand::SignSetup => {
            let status = Process::new("git")?
                .args(["config", "commit.gpgsign", "true"])
                .status()?;
            if status.success() {
                println!("git commit signing enabled");
            }
        }
        GitCommand::RemoteSetup {
            provider,
            visibility,
            token_env: _,
        } => {
            let provider = provider.as_deref().unwrap_or("github");
            println!(
                "remote setup: provider={provider} visibility={}",
                visibility.as_deref().unwrap_or("public")
            );
            println!("run `git remote add origin <url>` to set the remote");
        }
    }
    Ok(())
}
