use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{LodeError, Process, Result};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct PluginSecurity {
    pub network: bool,
    pub execute: bool,
    pub fs_write: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginInstallReceipt {
    pub schema_version: u32,
    pub source: String,
    pub installed_at: String,
    pub reviewed: bool,
    pub allow_unsafe: bool,
    pub permissions: PluginSecurity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEvent {
    PreCommit,
    PostCommit,
    PrePush,
    PostCheckout,
    PreSave,
    PostSave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookConfig {
    pub hooks_dir: Option<Utf8PathBuf>,
    pub enabled: bool,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            hooks_dir: None,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hook {
    pub event: HookEvent,
    pub path: Utf8PathBuf,
    pub source: String,
    pub runtime: HookRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookRuntime {
    Shell,
    Node,
    Python,
    Binary(Utf8PathBuf),
}

pub fn discover_hooks(project_dir: &Utf8Path, config: &HookConfig) -> Result<Vec<Hook>> {
    if !config.enabled {
        return Ok(Vec::new());
    }
    let hooks_dir = config
        .hooks_dir
        .clone()
        .unwrap_or_else(|| project_dir.join(".lode").join("hooks"));
    if !hooks_dir.exists() {
        return Ok(Vec::new());
    }
    let mut hooks = Vec::new();
    visit_hooks_dir(&hooks_dir, &mut hooks)?;
    Ok(hooks)
}

fn visit_hooks_dir(dir: &Utf8Path, hooks: &mut Vec<Hook>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: PathBuf::from(dir.as_str()),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: PathBuf::from(dir.as_str()),
            source,
        })?;
        let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if child.is_dir() {
            visit_hooks_dir(&child, hooks)?;
            continue;
        }
        if let Some(hook) = classify_hook(&child) {
            hooks.push(hook);
        }
    }
    Ok(())
}

fn classify_hook(path: &Utf8Path) -> Option<Hook> {
    let file_name = path.file_name()?;
    let (event, runtime) = if file_name.starts_with("pre-commit") {
        (HookEvent::PreCommit, HookRuntime::Shell)
    } else if file_name.starts_with("post-commit") {
        (HookEvent::PostCommit, HookRuntime::Shell)
    } else if file_name.starts_with("pre-push") {
        (HookEvent::PrePush, HookRuntime::Shell)
    } else if file_name.starts_with("post-checkout") {
        (HookEvent::PostCheckout, HookRuntime::Shell)
    } else if file_name.starts_with("pre-save") {
        (HookEvent::PreSave, HookRuntime::Shell)
    } else if file_name.starts_with("post-save") {
        (HookEvent::PostSave, HookRuntime::Shell)
    } else {
        return None;
    };
    let source = fs::read_to_string(path).ok().unwrap_or_default();
    Some(Hook {
        event,
        path: path.to_path_buf(),
        source,
        runtime,
    })
}

fn run_hook_program(
    program: &str,
    args: &[&str],
    hook: &Hook,
    project_dir: &Utf8Path,
) -> Result<()> {
    let status = Process::new(program)?
        .args(args)
        .current_dir(project_dir.as_std_path())
        .status()?;
    if !status.success() {
        return Err(LodeError::Message(format!(
            "hook '{}' failed with exit code {:?}",
            hook.path.file_name().unwrap_or("unknown"),
            status.code()
        )));
    }
    Ok(())
}

fn enforce_hook_execute_permission(hook: &Hook) -> Result<()> {
    let hook_str = hook.path.as_str();
    if !hook_str.contains("plugins") {
        return Ok(());
    }
    let plugins_dir = crate::install::global_asset_dir("plugins")?;
    let plugins_str = plugins_dir.as_str();
    if !hook_str.starts_with(plugins_str) {
        return Ok(());
    }
    let relative = &hook_str[plugins_str.len() + 1..];
    let plugin_name = relative.split(['/', '\\']).next().unwrap_or("");
    if plugin_name.is_empty() {
        return Ok(());
    }
    let plugin_path = plugins_dir.join(plugin_name);
    let receipt_path = plugin_path.join(".lode-install.json");
    if !receipt_path.exists() {
        return Err(LodeError::Message(format!(
            "Plugin '{}' has no install receipt; refusing to execute hook",
            plugin_name
        )));
    }
    let raw = std::fs::read_to_string(&receipt_path).map_err(|source| LodeError::Io {
        path: receipt_path.as_str().into(),
        source,
    })?;
    let receipt: crate::PluginInstallReceipt = serde_json::from_str(&raw)
        .map_err(|e| LodeError::Message(format!("invalid plugin receipt: {e}")))?;
    if !receipt.permissions.execute && !receipt.allow_unsafe {
        return Err(LodeError::Message(format!(
            "Plugin '{}' does not declare execute permission; hook execution blocked",
            plugin_name
        )));
    }
    Ok(())
}

fn check_project_hook_permission(hook: &Hook) -> Result<()> {
    let hook_str = hook.path.as_str();
    let plugins_dir = crate::install::global_asset_dir("plugins")?;
    if hook_str.starts_with(plugins_dir.as_str()) {
        return enforce_hook_execute_permission(hook);
    }
    Ok(())
}

pub fn run_hook(hook: &Hook, project_dir: &Utf8Path, dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }
    check_project_hook_permission(hook)?;
    match &hook.runtime {
        HookRuntime::Shell => {
            let (shell, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
                let ext = hook.path.extension().unwrap_or("");
                if ext.eq_ignore_ascii_case("bat") || ext.eq_ignore_ascii_case("cmd") {
                    ("cmd", &["/c", hook.path.as_str()])
                } else {
                    ("sh", &[hook.path.as_str()])
                }
            } else {
                ("sh", &[hook.path.as_str()])
            };
            run_hook_program(shell, args, hook, project_dir)?;
        }
        HookRuntime::Node => {
            run_hook_program("node", &[hook.path.as_str()], hook, project_dir)?;
        }
        HookRuntime::Python => {
            run_hook_program("python", &[hook.path.as_str()], hook, project_dir)?;
        }
        HookRuntime::Binary(binary_path) => {
            run_hook_program(
                binary_path.as_str(),
                &[hook.path.as_str()],
                hook,
                project_dir,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_hooks_returns_empty_when_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let config = HookConfig {
            enabled: false,
            ..Default::default()
        };
        let hooks = discover_hooks(&root, &config).unwrap();
        assert!(hooks.is_empty());
    }

    #[test]
    fn discover_hooks_returns_empty_when_no_hooks_dir() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let config = HookConfig::default();
        let hooks = discover_hooks(&root, &config).unwrap();
        assert!(hooks.is_empty());
    }

    #[test]
    fn discover_hooks_finds_shell_hooks() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let hooks_dir = root.join(".lode").join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        fs::write(hooks_dir.join("pre-commit.sh"), "#!/bin/sh\necho hello").unwrap();
        fs::write(hooks_dir.join("post-save.sh"), "#!/bin/sh\necho done").unwrap();

        let config = HookConfig::default();
        let hooks = discover_hooks(&root, &config).unwrap();
        assert_eq!(hooks.len(), 2);
        assert!(hooks.iter().any(|h| h.event == HookEvent::PreCommit));
        assert!(hooks.iter().any(|h| h.event == HookEvent::PostSave));
    }

    #[test]
    fn run_hook_dry_run_skips_execution() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let hook = Hook {
            event: HookEvent::PreCommit,
            path: root.join("pre-commit.sh"),
            source: String::new(),
            runtime: HookRuntime::Shell,
        };
        run_hook(&hook, &root, true).unwrap();
    }
}
