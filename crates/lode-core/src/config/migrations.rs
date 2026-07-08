use std::fs;
use std::path::PathBuf;

use camino::Utf8PathBuf;

use crate::config;
use crate::fs_safety::ValidatedRoot;
use crate::install::trusted_root;
use crate::{LodeError, Result};

pub fn migrate_config_source_if_needed(path: &Utf8PathBuf, raw: &str) -> Result<String> {
    let mut value: toml::Value =
        toml::from_str(raw).map_err(|source| LodeError::TomlDeserialize {
            path: PathBuf::from(path.as_str()),
            source,
        })?;
    let schema_version = value
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        .unwrap_or(0);
    let schema_version = u32::try_from(schema_version).unwrap_or(0);

    if schema_version == config::SCHEMA_VERSION {
        return Ok(raw.to_string());
    }
    if schema_version > config::SCHEMA_VERSION {
        return Err(LodeError::SchemaMismatch {
            expected: config::SCHEMA_VERSION,
            found: schema_version,
        });
    }

    let backup_path = backup_config(path, schema_version, raw)?;
    let mut defaults = toml::Value::try_from(config::default_config())?;
    merge_toml_defaults(&mut defaults, value);
    value = defaults;
    if let Some(table) = value.as_table_mut() {
        table.insert(
            "schema_version".to_string(),
            toml::Value::Integer(i64::from(config::SCHEMA_VERSION)),
        );
    }
    let migrated = toml::to_string_pretty(&value)?;
    let parent = path
        .parent()
        .ok_or_else(|| LodeError::Message("global config path must have a parent".into()))?;
    trusted_root(parent)?.write_atomic(
        path.file_name()
            .ok_or_else(|| LodeError::Message("global config path must name a file".into()))?,
        &migrated,
    )?;
    prune_config_backups(path)?;
    eprintln!(
        "config migrated: schema v{} -> v{}; backup: {}",
        schema_version,
        config::SCHEMA_VERSION,
        backup_path
    );
    Ok(migrated)
}

fn merge_toml_defaults(defaults: &mut toml::Value, existing: toml::Value) {
    match (defaults, existing) {
        (toml::Value::Table(defaults), toml::Value::Table(existing)) => {
            for (key, value) in existing {
                if key == "schema_version" {
                    continue;
                }
                match defaults.get_mut(&key) {
                    Some(default_value) => merge_toml_defaults(default_value, value),
                    None => {
                        defaults.insert(key, value);
                    }
                }
            }
        }
        (defaults, existing) => *defaults = existing,
    }
}

fn backup_config(path: &Utf8PathBuf, schema_version: u32, raw: &str) -> Result<Utf8PathBuf> {
    let backup_path = Utf8PathBuf::from(format!("{}.bak-schema-{}", path, schema_version));
    let parent = backup_path
        .parent()
        .ok_or_else(|| LodeError::Message("backup path must have a parent".into()))?;
    trusted_root(parent)?.write_atomic(
        backup_path
            .file_name()
            .ok_or_else(|| LodeError::Message("backup path must name a file".into()))?,
        raw,
    )?;
    Ok(backup_path)
}

fn prune_config_backups(path: &Utf8PathBuf) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let Some(name) = path.file_name() else {
        return Ok(());
    };
    let prefix = format!("{name}.bak-schema-");
    let mut backups = Vec::new();
    let entries = fs::read_dir(parent).map_err(|source| LodeError::Io {
        path: PathBuf::from(parent.as_str()),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| LodeError::Io {
            path: PathBuf::from(parent.as_str()),
            source,
        })?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_name.starts_with(&prefix) {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            backups.push((
                modified,
                Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                    LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
                })?,
            ));
        }
    }
    backups.sort_by(|left, right| right.0.cmp(&left.0));
    let root = ValidatedRoot::new(parent)?;
    for (_, backup) in backups.into_iter().skip(5) {
        root.remove_file(
            backup
                .file_name()
                .ok_or_else(|| LodeError::Message("backup path must name a file".into()))?,
        )?;
    }
    Ok(())
}
