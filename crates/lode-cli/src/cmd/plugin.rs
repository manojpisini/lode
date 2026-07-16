#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{global_asset_dir, LodeError, PluginInstallReceipt, PluginSecurity, ValidatedRoot};
use serde::{Deserialize, Serialize};

use crate::PluginCommand;

pub(crate) fn plugin_command(command: PluginCommand) -> lode_core::Result<()> {
    match command {
        PluginCommand::List => crate::list_dir(global_asset_dir("plugins")?)?,
        PluginCommand::Search { query, output } => {
            let entries = search_plugin_index(query.as_deref())?;
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&entries)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                if entries.is_empty() {
                    println!("no plugins found");
                } else {
                    for entry in entries {
                        println!(
                            "{}\t{}\t{}\t{}",
                            entry.name,
                            entry.version,
                            if entry.installed {
                                "installed"
                            } else {
                                "available"
                            },
                            entry.description
                        );
                    }
                }
            }
        }
        PluginCommand::Add {
            source,
            allow_unsafe,
        } => {
            if !source.exists() || !source.is_dir() {
                return Err(LodeError::Message(format!(
                    "plugin source must be a directory: {source}"
                )));
            }
            let entry = require_plugin_manifest(&source)?;
            crate::safe_relative_path(&entry.name)?;
            enforce_plugin_permissions(&source, allow_unsafe)?;
            let name = entry.name;
            let plugins_dir = global_asset_dir("plugins")?;
            let plugins_root = ValidatedRoot::new(&plugins_dir)?;
            let destination = plugins_dir.join(&name);
            if destination.exists() {
                return Err(LodeError::Message(format!("plugin already exists: {name}")));
            }
            plugins_root.create_dir_all(&name)?;
            let source_root = ValidatedRoot::new(&source)?;
            copy_dir_recursive(&source_root, &plugins_root, "", &name)?;
            write_plugin_install_receipt(&destination, &source, allow_unsafe)?;
            println!("added plugin {name}");
        }
        PluginCommand::Remove { name } => {
            let plugins_dir = global_asset_dir("plugins")?;
            let plugins_root = ValidatedRoot::new(&plugins_dir)?;
            let relative = crate::safe_relative_path(&name)?;
            let path = plugins_dir.join(&relative);
            if !path.exists() {
                return Err(LodeError::Message(format!("plugin not found: {name}")));
            }
            plugins_root.remove_dir_all(relative)?;
            println!("removed plugin {name}");
        }
        PluginCommand::Update { name } => {
            if let Some(name) = name {
                let path = global_asset_dir("plugins")?.join(crate::safe_relative_path(&name)?);
                if !path.exists() {
                    return Err(LodeError::Message(format!("plugin not found: {name}")));
                }
                println!("plugin {name} is local; refresh by re-adding from source");
            } else {
                println!("local plugins checked");
            }
        }
        PluginCommand::Info { name } => {
            let path = global_asset_dir("plugins")?.join(crate::safe_relative_path(&name)?);
            if !path.exists() {
                return Err(LodeError::Message(format!("plugin not found: {name}")));
            }
            let entry = plugin_index_entry(&path, true)?;
            println!("name\t{}", entry.name);
            println!("version\t{}", entry.version);
            println!("description\t{}", entry.description);
            println!("path\t{path}");
            for child in ["templates", "profiles", "snippets", "recipes", "commands"] {
                println!("{child}\t{}", crate::status_bool(path.join(child).exists()));
            }
            if !entry.capabilities.is_empty() {
                println!("capabilities\t{}", entry.capabilities.join(","));
            }
            let security = read_plugin_security(&path)?;
            println!("network\t{}", crate::status_bool(security.network));
            println!("execute\t{}", crate::status_bool(security.execute));
            if !security.fs_write.is_empty() {
                println!("fs_write\t{}", security.fs_write.join(","));
            }
            if let Some(receipt) = read_plugin_install_receipt(&path)? {
                println!("trusted\t{}", crate::status_bool(receipt.reviewed));
                println!("installed_at\t{}", receipt.installed_at);
                println!("installed_from\t{}", receipt.source);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginIndexEntry {
    name: String,
    version: String,
    description: String,
    source: String,
    installed: bool,
    path: Option<Utf8PathBuf>,
    capabilities: Vec<String>,
}

// PluginSecurity and PluginInstallReceipt are now defined in lode-core (hooks.rs)

fn require_plugin_manifest(source: &Utf8PathBuf) -> lode_core::Result<PluginIndexEntry> {
    let manifest = source.join("plugin.toml");
    if !manifest.exists() {
        return Err(LodeError::Message(
            "plugin manifest required: plugin.toml".to_string(),
        ));
    }
    plugin_index_entry(source, false)
}

fn enforce_plugin_permissions(source: &Utf8PathBuf, allow_unsafe: bool) -> lode_core::Result<()> {
    let security = read_plugin_security(source)?;
    for path in &security.fs_write {
        crate::safe_relative_path(path)?;
    }
    let has_executable_surface = source.join("bin").exists() || source.join("hooks").exists();
    if has_executable_surface && !security.execute {
        return Err(LodeError::Message(
            "plugin contains bin/ or hooks/ but does not declare permissions.execute = true"
                .to_string(),
        ));
    }
    let unsafe_reasons = [
        (security.network, "network"),
        (security.execute || has_executable_surface, "execute"),
    ]
    .into_iter()
    .filter_map(|(enabled, reason)| enabled.then_some(reason))
    .collect::<Vec<_>>();
    if !unsafe_reasons.is_empty() && !allow_unsafe {
        return Err(LodeError::Message(format!(
            "plugin requests unsafe permission(s): {}; rerun with --allow-unsafe after review",
            unsafe_reasons.join(",")
        )));
    }
    Ok(())
}

pub(crate) fn read_plugin_security(path: &Utf8PathBuf) -> lode_core::Result<PluginSecurity> {
    let manifest = path.join("plugin.toml");
    if !manifest.exists() {
        return Ok(PluginSecurity::default());
    }
    let raw = fs::read_to_string(&manifest).map_err(|source| LodeError::Io {
        path: manifest.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    let Some(permissions) = value.get("permissions") else {
        return Ok(PluginSecurity::default());
    };
    let network = permissions
        .get("network")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let execute = permissions
        .get("execute")
        .or_else(|| permissions.get("fs_execute"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let fs_write = permissions
        .get("fs_write")
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(PluginSecurity {
        network,
        execute,
        fs_write,
    })
}

fn write_plugin_install_receipt(
    destination: &Utf8PathBuf,
    source: &Utf8PathBuf,
    allow_unsafe: bool,
) -> lode_core::Result<()> {
    let receipt = PluginInstallReceipt {
        schema_version: 3,
        source: source.to_string(),
        installed_at: crate::now_timestamp(),
        reviewed: true,
        allow_unsafe,
        permissions: read_plugin_security(destination)?,
    };
    let raw = serde_json::to_string_pretty(&receipt)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    ValidatedRoot::new(destination)?.write_atomic(".lode-install.json", raw)?;
    Ok(())
}

pub(crate) fn read_plugin_install_receipt(
    path: &Utf8PathBuf,
) -> lode_core::Result<Option<PluginInstallReceipt>> {
    let receipt = path.join(".lode-install.json");
    if !receipt.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&receipt).map_err(|source| LodeError::Io {
        path: receipt.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|error| LodeError::Message(error.to_string()))
}

fn search_plugin_index(query: Option<&str>) -> lode_core::Result<Vec<PluginIndexEntry>> {
    let mut entries = default_plugin_registry();
    let plugins_dir = global_asset_dir("plugins")?;
    if plugins_dir.exists() {
        for entry in fs::read_dir(&plugins_dir).map_err(|source| LodeError::Io {
            path: plugins_dir.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: plugins_dir.as_str().into(),
                source,
            })?;
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            if path.is_dir() {
                let installed = plugin_index_entry(&path, true)?;
                entries.retain(|candidate| candidate.name != installed.name);
                entries.push(installed);
            }
        }
    }

    if let Some(query) = query {
        let query = query.to_ascii_lowercase();
        entries.retain(|entry| {
            entry.name.to_ascii_lowercase().contains(&query)
                || entry.description.to_ascii_lowercase().contains(&query)
                || entry
                    .capabilities
                    .iter()
                    .any(|capability| capability.to_ascii_lowercase().contains(&query))
        });
    }
    entries.sort_by(|left, right| {
        right
            .installed
            .cmp(&left.installed)
            .then(left.name.cmp(&right.name))
    });
    Ok(entries)
}

fn plugin_index_entry(path: &Utf8PathBuf, installed: bool) -> lode_core::Result<PluginIndexEntry> {
    let manifest = path.join("plugin.toml");
    let fallback_name = path
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "plugin".to_string());
    let mut entry = PluginIndexEntry {
        name: fallback_name,
        version: "0.0.0".to_string(),
        description: "Local Lode plugin".to_string(),
        source: "local".to_string(),
        installed,
        path: Some(path.clone()),
        capabilities: plugin_capabilities(path),
    };
    if manifest.exists() {
        let raw = fs::read_to_string(&manifest).map_err(|source| LodeError::Io {
            path: manifest.as_str().into(),
            source,
        })?;
        let value: toml::Value =
            toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
        let plugin = value.get("plugin").unwrap_or(&value);
        if let Some(name) = plugin.get("name").and_then(toml::Value::as_str) {
            entry.name = name.to_string();
        }
        if let Some(version) = plugin.get("version").and_then(toml::Value::as_str) {
            entry.version = version.to_string();
        }
        if let Some(description) = plugin.get("description").and_then(toml::Value::as_str) {
            entry.description = description.to_string();
        }
    }
    Ok(entry)
}

fn plugin_capabilities(path: &Utf8PathBuf) -> Vec<String> {
    [
        "templates",
        "profiles",
        "snippets",
        "recipes",
        "commands",
        "hooks",
        "bin",
    ]
    .iter()
    .filter(|name| path.join(name).exists())
    .map(|name| (*name).to_string())
    .collect()
}

fn default_plugin_registry() -> Vec<PluginIndexEntry> {
    [
        (
            "lode-plugin-tauri",
            "desktop and Tauri scaffolding, commands, and checks",
            &["templates", "commands", "recipes"][..],
        ),
        (
            "lode-plugin-minecraft",
            "Minecraft Fabric, Forge, NeoForge, and Paper project helpers",
            &["templates", "snippets", "commands"][..],
        ),
        (
            "lode-plugin-competitive",
            "competitive programming templates, runners, and snippets",
            &["templates", "snippets", "commands"][..],
        ),
        (
            "lode-plugin-agent-pack",
            "agent context packs for Claude, Codex, Cursor, and Windsurf",
            &["templates", "commands", "hooks"][..],
        ),
    ]
    .into_iter()
    .map(|(name, description, capabilities)| PluginIndexEntry {
        name: name.to_string(),
        version: "registry".to_string(),
        description: description.to_string(),
        source: "builtin-index".to_string(),
        installed: false,
        path: None,
        capabilities: capabilities
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    })
    .collect()
}

fn copy_dir_recursive(
    source_root: &ValidatedRoot,
    destination_root: &ValidatedRoot,
    source_relative: &str,
    destination_relative: &str,
) -> lode_core::Result<()> {
    destination_root.create_dir_all(destination_relative)?;
    let source = source_root.resolve(source_relative)?;
    for entry in fs::read_dir(&source).map_err(|source_error| LodeError::Io {
        path: source.to_string_lossy().into_owned().into(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| LodeError::Io {
            path: source.to_string_lossy().into_owned().into(),
            source: source_error,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let child_source =
            crate::safe_relative_path(Utf8PathBuf::from(source_relative).join(&name).as_str())?;
        let child_destination = crate::safe_relative_path(
            Utf8PathBuf::from(destination_relative).join(&name).as_str(),
        )?;
        let source_path = source_root.resolve(&child_source)?;
        if source_path.is_dir() {
            copy_dir_recursive(
                source_root,
                destination_root,
                child_source.as_str(),
                child_destination.as_str(),
            )?;
        } else {
            let contents = fs::read(&source_path).map_err(|source_error| LodeError::Io {
                path: source_path.to_string_lossy().into_owned().into(),
                source: source_error,
            })?;
            destination_root.write_atomic(child_destination, contents)?;
        }
    }
    Ok(())
}
