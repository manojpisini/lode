#![deny(unsafe_code)]

use lode_core::{
    diff_locks, load_lock, lockfile_path, new_lock, save_lock, update_lock, verify_lock,
    LockAssetEntry, LodeError,
};

use crate::{LockCommand, OutputFormat};

pub(crate) fn lock_command(command: LockCommand) -> lode_core::Result<()> {
    match command {
        LockCommand::Show { output } => lock_show(output),
        LockCommand::Verify { output } => lock_verify(output),
        LockCommand::Update { id, output } => lock_update(id, output),
        LockCommand::Diff { output } => lock_diff(output),
    }
}

fn project_dir() -> lode_core::Result<camino::Utf8PathBuf> {
    let dir = std::env::current_dir().map_err(|e| LodeError::Message(e.to_string()))?;
    camino::Utf8PathBuf::try_from(dir).map_err(|_| LodeError::Message("invalid path".to_string()))
}

fn lock_or_new(dir: &camino::Utf8PathBuf) -> (camino::Utf8PathBuf, lode_core::LodeLock) {
    let path = lockfile_path(dir);
    let lock = if path.exists() {
        load_lock(&path).unwrap_or_else(|_| new_lock())
    } else {
        new_lock()
    };
    (path, lock)
}

fn lock_show(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let path = lockfile_path(&dir);

    if !path.exists() {
        if output.should_use_json() {
            println!("{{}}");
        } else {
            println!("no lockfile found at {path}");
        }
        return Ok(());
    }

    let lock = load_lock(&path)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&lock).map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Lode Lock v{}", lock.schema_version);
        println!("  created: {}", lock.created_at);
        println!("  updated: {}", lock.updated_at);
        println!("  assets ({}):", lock.assets.len());
        for entry in &lock.assets {
            println!("    {} v{}", entry.id, entry.version);
            if let Some(ref kind) = entry.kind {
                println!("      kind: {kind}");
            }
            if let Some(ref path) = entry.path {
                println!("      path: {path}");
            }
            println!("      sha256: {}", &entry.sha256[..16]);
        }
    }
    Ok(())
}

fn lock_verify(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let path = lockfile_path(&dir);

    if !path.exists() {
        return Err(LodeError::Message("no lockfile found -- run `lode lock update` first".to_string()));
    }

    let lock = load_lock(&path)?;
    let report = verify_lock(&lock);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if report.valid {
            println!("lockfile verified ({} entries checked, {} warnings)", report.checked, report.warnings.len());
            for w in &report.warnings {
                println!("  warning: {w}");
            }
        } else {
            println!("lockfile verification FAILED ({} errors)", report.errors.len());
            for e in &report.errors {
                println!("  error: {e}");
            }
            for w in &report.warnings {
                println!("  warning: {w}");
            }
        }
    }
    Ok(())
}

fn lock_update(
    ids: Option<Vec<String>>,
    output: OutputFormat,
) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let (path, mut lock) = lock_or_new(&dir);

    // Collect current assets to lock
    let mut new_assets = Vec::new();

    // Check scaffold.lock for template-generated files
    let scaffold_path = dir.join(".lode/scaffold.lock");
    if scaffold_path.exists() {
        if let Ok(scaffold_lock) = lode_core::load_scaffold_lock(&dir) {
            for entry in &scaffold_lock.entries {
                let full_path = dir.join(&entry.destination);
                let hash = lode_core::hash_file(&full_path).unwrap_or_default();
                new_assets.push(LockAssetEntry {
                    id: format!("template://{}", entry.template),
                    version: "1.0.0".to_string(),
                    sha256: hash,
                    kind: Some("template".to_string()),
                    path: Some(entry.destination.as_str().to_string()),
                });
            }
        }
    }

    // Check project config for declared assets
    let config_path = dir.join(".lode/project.toml");
    if config_path.exists() {
        if let Ok(config) = lode_core::load_project_config(&dir) {
            if let Some(ref assets) = config.project.assets {
                for asset_id in assets {
                    if let Some(ref filter_ids) = ids {
                        if !filter_ids.iter().any(|f| asset_id.contains(f)) {
                            continue;
                        }
                    }
                    if !new_assets.iter().any(|a| a.id == *asset_id) {
                        new_assets.push(LockAssetEntry {
                            id: asset_id.to_string(),
                            version: "1.0.0".to_string(),
                            sha256: String::new(),
                            kind: Some("asset".to_string()),
                            path: None,
                        });
                    }
                }
            }
        }
    }

    let diff = update_lock(&mut lock, new_assets);
    save_lock(&path, &lock)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&diff)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("lockfile updated at {path}");
        if !diff.added.is_empty() {
            println!("  added ({}):", diff.added.len());
            for a in &diff.added {
                println!("    + {} v{}", a.id, a.version);
            }
        }
        if !diff.removed.is_empty() {
            println!("  removed ({}):", diff.removed.len());
            for a in &diff.removed {
                println!("    - {} v{}", a.id, a.version);
            }
        }
        if !diff.changed.is_empty() {
            println!("  changed ({}):", diff.changed.len());
            for (old, new) in &diff.changed {
                println!("    ~ {} {} -> {}", old.id, old.version, new.version);
            }
        }
        if diff.unchanged > 0 {
            println!("  unchanged: {}", diff.unchanged);
        }
    }
    Ok(())
}

fn lock_diff(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let path = lockfile_path(&dir);

    if !path.exists() {
        return Err(LodeError::Message("no lockfile found -- run `lode lock update` first".to_string()));
    }

    let lock = load_lock(&path)?;
    let expected = new_lock();

    let diff = diff_locks(&lock, &expected);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&diff)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if diff.added.is_empty() && diff.removed.is_empty() && diff.changed.is_empty() {
            println!("lockfile is up to date");
        } else {
            println!("lockfile diff:");
            if !diff.added.is_empty() {
                for a in &diff.added {
                    println!("  + {} v{}", a.id, a.version);
                }
            }
            if !diff.removed.is_empty() {
                for a in &diff.removed {
                    println!("  - {} v{}", a.id, a.version);
                }
            }
            if !diff.changed.is_empty() {
                for (old, new) in &diff.changed {
                    println!("  ~ {} {} -> {}", old.id, old.version, new.version);
                }
            }
        }
    }
    Ok(())
}
