#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::{fs, io};

use camino::Utf8PathBuf;
use lode_core::{global_asset_dir, global_dir, LodeError};

use crate::HooksCommand;

pub(crate) fn hooks(command: HooksCommand) -> lode_core::Result<()> {
    match command {
        HooksCommand::List => {
            for hook in discover_hooks()? {
                println!("{}\t{}\t{}", hook.event, hook.source, hook.path);
            }
        }
        HooksCommand::Status => {
            hooks_status()?;
            let hooks = discover_hooks()?;
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for hook in hooks {
                *counts.entry(hook.event).or_default() += 1;
            }
            for (event, count) in counts {
                println!("{event}\t{count} hook(s)");
            }
        }
        HooksCommand::Test { event } => test_hook(&event)?,
        HooksCommand::Run { event, dry_run } => run_hooks(&event, dry_run)?,
    }
    Ok(())
}

pub(crate) fn hooks_status() -> lode_core::Result<()> {
    let hooks_dir = Utf8PathBuf::from(".git").join("hooks");
    for name in ["pre-commit", "pre-push"] {
        let path = hooks_dir.join(name);
        let status = if path.exists()
            && fs::read_to_string(&path)
                .unwrap_or_default()
                .contains("lode-managed")
        {
            "managed"
        } else {
            "missing"
        };
        println!("{name}\t{status}");
    }
    Ok(())
}

fn test_hook(event: &str) -> lode_core::Result<()> {
    let mut hooks = discover_hooks()?;
    hooks.retain(|hook| hook.event == event);
    if hooks.is_empty() {
        let script = match event {
            "pre-commit" => "lode check . && lode scan secrets .",
            "pre-push" => "lode task test",
            other => return Err(LodeError::Message(format!("unknown hook event: {other}"))),
        };
        println!("hook {event}: {script}");
        return Ok(());
    }
    println!("hook execution plan for {event}:");
    for hook in hooks {
        println!("{}\t{}\t{}", hook.source, hook.runtime, hook.path);
    }
    Ok(())
}

fn run_hooks(event: &str, dry_run: bool) -> lode_core::Result<()> {
    let mut hooks = discover_hooks()?;
    hooks.retain(|hook| hook.event == event);
    if hooks.is_empty() {
        return Err(LodeError::Message(format!(
            "no hooks found for event: {event}"
        )));
    }
    for hook in hooks {
        let (program, args) = hook_command(&hook)?;
        if dry_run {
            println!(
                "would run hook {}\t{}\t{} {}",
                hook.source,
                hook.runtime,
                program,
                args.join(" ")
            );
            continue;
        }
        println!(
            "running hook {}\t{}\t{}",
            hook.source, hook.runtime, hook.path
        );
        let before = plugin_hook_file_snapshot(&hook)?;
        let envs = hook_runtime_env(&hook);
        let status = crate::run_process_status_with_env(program, &args, None, &envs)?;
        if !status.success() {
            return Err(LodeError::Message(format!(
                "hook {} {} failed with {status}",
                hook.source, hook.path
            )));
        }
        enforce_plugin_hook_writes(&hook, before)?;
    }
    Ok(())
}

fn hook_command(hook: &DiscoveredHook) -> lode_core::Result<(&'static str, Vec<String>)> {
    let path = hook.path.to_string();
    match hook.runtime.as_str() {
        "powershell" => Ok((
            "powershell",
            vec!["-NoProfile".to_string(), "-File".to_string(), path],
        )),
        "python" => Ok(("python", vec![path])),
        "node" => Ok(("node", vec![path])),
        "lua" => Ok(("lua", vec![path])),
        "sh" => Ok(("sh", vec![path])),
        other => Err(LodeError::Message(format!(
            "unsupported hook runtime: {other}"
        ))),
    }
}

fn hook_runtime_env(hook: &DiscoveredHook) -> Vec<(&'static str, String)> {
    let mut envs = vec![
        ("LODE_HOOK_EVENT", hook.event.clone()),
        ("LODE_HOOK_SOURCE", hook.source.clone()),
        ("LODE_HOOK_RUNTIME", hook.runtime.clone()),
    ];
    if let Some(plugin) = hook.source.strip_prefix("plugin:") {
        let security = hook.plugin_security.clone().unwrap_or_default();
        envs.push(("LODE_PLUGIN_NAME", plugin.to_string()));
        envs.push(("LODE_PLUGIN_ALLOW_NETWORK", security.network.to_string()));
        envs.push(("LODE_PLUGIN_ALLOW_EXECUTE", security.execute.to_string()));
        envs.push(("LODE_PLUGIN_FS_WRITE", security.fs_write.join(";")));
    }
    envs
}

type HookFileSnapshot = BTreeMap<String, String>;

fn plugin_hook_file_snapshot(hook: &DiscoveredHook) -> lode_core::Result<Option<HookFileSnapshot>> {
    if hook.plugin_security.is_none() {
        return Ok(None);
    }
    snapshot_project_contents(&crate::current_dir()?).map(Some)
}

fn enforce_plugin_hook_writes(
    hook: &DiscoveredHook,
    before: Option<HookFileSnapshot>,
) -> lode_core::Result<()> {
    let Some(before) = before else {
        return Ok(());
    };
    let security = hook.plugin_security.clone().unwrap_or_default();
    let after = snapshot_project_contents(&crate::current_dir()?)?;
    let changed = changed_snapshot_paths(&before, &after);
    let denied = changed
        .into_iter()
        .filter(|path| !plugin_write_allowed(path, &security.fs_write))
        .collect::<Vec<_>>();
    if !denied.is_empty() {
        return Err(LodeError::Message(format!(
            "plugin hook {} wrote outside declared fs_write paths: {}",
            hook.source,
            denied.join(",")
        )));
    }
    Ok(())
}

fn snapshot_project_contents(root: &Utf8PathBuf) -> lode_core::Result<HookFileSnapshot> {
    let mut snapshot = BTreeMap::new();
    snapshot_contents_dir(root, root, &mut snapshot)?;
    Ok(snapshot)
}

fn snapshot_contents_dir(
    root: &Utf8PathBuf,
    dir: &Utf8PathBuf,
    snapshot: &mut HookFileSnapshot,
) -> lode_core::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(LodeError::Io {
                path: dir.as_str().into(),
                source,
            })
        }
    };

    for entry in entries {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|path| LodeError::Message(format!("non-utf8 path: {}", path.display())))?;
        let name = path.file_name().unwrap_or_default();
        if crate::should_skip_watch_path(name) {
            continue;
        }
        let metadata = entry.metadata().map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        if metadata.is_dir() {
            snapshot_contents_dir(root, &path, snapshot)?;
        } else if metadata.is_file() {
            let contents = fs::read(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let relative = path
                .strip_prefix(root)
                .map(|path| path.as_str().replace('\\', "/"))
                .unwrap_or_else(|_| path.as_str().replace('\\', "/"));
            snapshot.insert(relative, crate::content_hash_bytes(&contents));
        }
    }
    Ok(())
}

fn changed_snapshot_paths(before: &HookFileSnapshot, after: &HookFileSnapshot) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|path| {
            if before.get(path) == after.get(path) {
                None
            } else {
                Some(path.clone())
            }
        })
        .collect()
}

fn plugin_write_allowed(path: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|allowed| {
        let allowed = allowed
            .trim_end_matches("/**")
            .trim_end_matches("/*")
            .trim_end_matches('/');
        path == allowed || path.starts_with(&format!("{allowed}/"))
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveredHook {
    event: String,
    source: String,
    runtime: String,
    path: Utf8PathBuf,
    plugin_security: Option<lode_core::PluginSecurity>,
}

pub(crate) fn discover_hooks() -> lode_core::Result<Vec<DiscoveredHook>> {
    let mut hooks = Vec::new();
    discover_plugin_hooks(&mut hooks)?;
    discover_hook_dir("global", &global_dir()?.join("hooks"), &mut hooks)?;
    discover_hook_dir(
        "project",
        &Utf8PathBuf::from(".lode").join("hooks"),
        &mut hooks,
    )?;
    hooks.sort_by(|left, right| {
        hook_source_rank(&left.source)
            .cmp(&hook_source_rank(&right.source))
            .then(left.event.cmp(&right.event))
            .then(left.path.cmp(&right.path))
    });
    Ok(hooks)
}

fn discover_plugin_hooks(hooks: &mut Vec<DiscoveredHook>) -> lode_core::Result<()> {
    let root = global_asset_dir("plugins")?;
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&root).map_err(|source| LodeError::Io {
        path: root.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: root.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if path.is_dir() {
            let name = path.file_name().unwrap_or("plugin");
            let hooks_dir = path.join("hooks");
            if hooks_dir.exists() {
                let security = require_plugin_runtime_permissions(name, &path)?;
                discover_hook_dir_with_security(
                    &format!("plugin:{name}"),
                    &hooks_dir,
                    Some(security),
                    hooks,
                )?;
            }
        }
    }
    Ok(())
}

fn require_plugin_runtime_permissions(
    name: &str,
    path: &Utf8PathBuf,
) -> lode_core::Result<lode_core::PluginSecurity> {
    let security = crate::read_plugin_security(path)?;
    for path in &security.fs_write {
        crate::safe_relative_path(path)?;
    }
    if !security.execute {
        return Err(LodeError::Message(format!(
            "plugin {name} has hooks but does not declare permissions.execute = true"
        )));
    }
    let Some(receipt) = crate::read_plugin_install_receipt(path)? else {
        return Err(LodeError::Message(format!(
            "plugin {name} has hooks but is missing install receipt; reinstall with `lode plugin add --allow-unsafe`"
        )));
    };
    if !receipt.reviewed || !receipt.allow_unsafe {
        return Err(LodeError::Message(format!(
            "plugin {name} has executable hooks but was not installed with reviewed unsafe permissions"
        )));
    }
    Ok(security)
}

fn discover_hook_dir(
    source: &str,
    dir: &Utf8PathBuf,
    hooks: &mut Vec<DiscoveredHook>,
) -> lode_core::Result<()> {
    discover_hook_dir_with_security(source, dir, None, hooks)
}

fn discover_hook_dir_with_security(
    source: &str,
    dir: &Utf8PathBuf,
    plugin_security: Option<lode_core::PluginSecurity>,
    hooks: &mut Vec<DiscoveredHook>,
) -> lode_core::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: dir.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if path.is_file() {
            if let Some((event, runtime)) = hook_file_event_runtime(&path) {
                hooks.push(DiscoveredHook {
                    event,
                    runtime,
                    source: source.to_string(),
                    path,
                    plugin_security: plugin_security.clone(),
                });
            }
        }
    }
    Ok(())
}

fn hook_file_event_runtime(path: &Utf8PathBuf) -> Option<(String, String)> {
    let file_name = path.file_name()?;
    let (event, runtime) = file_name
        .rsplit_once('.')
        .map(|(event, ext)| (event.to_string(), hook_runtime(ext)))
        .unwrap_or_else(|| (file_name.to_string(), "sh".to_string()));
    Some((event, runtime))
}

fn hook_runtime(extension: &str) -> String {
    match extension {
        "ps1" => "powershell",
        "py" => "python",
        "js" => "node",
        "lua" => "lua",
        _ => "sh",
    }
    .to_string()
}

fn hook_source_rank(source: &str) -> usize {
    match source {
        source if source.starts_with("plugin:") => 0,
        "global" => 1,
        "project" => 2,
        _ => 3,
    }
}
