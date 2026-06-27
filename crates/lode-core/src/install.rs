use std::{env, fs, path::PathBuf};

use camino::Utf8PathBuf;

use crate::{assets, config, fs_safety::ValidatedRoot, template::RenderContext, LodeError, Result};

const GLOBAL_DIRS: &[&str] = &[
    "profiles",
    "templates",
    "snippets",
    "licenses",
    "recipes",
    "plugins",
    "cache",
    "logs",
    "commands",
];

fn global_dir_env(child: &str) -> Option<&'static str> {
    match child {
        "templates" => Some("LODE_TEMPLATES"),
        "profiles" => Some("LODE_PROFILES"),
        "snippets" => Some("LODE_SNIPPETS"),
        "licenses" => Some("LODE_LICENSES"),
        "plugins" => Some("LODE_PLUGINS"),
        "recipes" => Some("LODE_RECIPES"),
        "commands" => Some("LODE_COMMANDS_DIR"),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupReport {
    pub global_dir: Utf8PathBuf,
    pub config_path: Utf8PathBuf,
    pub created_dirs: Vec<Utf8PathBuf>,
    pub wrote_config: bool,
}

pub fn global_dir() -> Result<Utf8PathBuf> {
    if let Ok(path) = env::var("LODE_CONFIG") {
        let path = Utf8PathBuf::from(path);
        return path.parent().map(Utf8PathBuf::from).ok_or_else(|| {
            LodeError::Message("LODE_CONFIG must include a parent directory".into())
        });
    }

    let home = dirs::home_dir()
        .ok_or_else(|| LodeError::Message("could not determine home directory".into()))?;
    let dir = home.join(".lode");
    Utf8PathBuf::from_path_buf(dir)
        .map_err(|path| LodeError::Message(format!("path is not valid UTF-8: {}", path.display())))
}

pub fn global_config_path() -> Result<Utf8PathBuf> {
    if let Ok(path) = env::var("LODE_CONFIG") {
        return Ok(Utf8PathBuf::from(path));
    }

    Ok(global_dir()?.join("config.toml"))
}

pub fn global_asset_dir(child: &str) -> Result<Utf8PathBuf> {
    if let Some(var) = global_dir_env(child) {
        if let Ok(path) = env::var(var) {
            return Ok(Utf8PathBuf::from(path));
        }
    }
    Ok(global_dir()?.join(child))
}

pub fn ensure_global_workspace() -> Result<()> {
    let dir = global_dir()?;
    let root = trusted_root(&dir)?;

    for child in GLOBAL_DIRS {
        let path = global_asset_dir(child)?;
        if path.starts_with(&dir) {
            root.create_dir_all(path.strip_prefix(&dir).expect("checked prefix"))?;
        } else {
            trusted_root(&path)?;
        }
    }

    Ok(())
}

pub fn setup_defaults(overwrite: bool) -> Result<SetupReport> {
    let dir = global_dir()?;
    let config_path = global_config_path()?;
    let mut created_dirs = Vec::new();

    if !dir.exists() {
        created_dirs.push(dir.clone());
    }
    let root = trusted_root(&dir)?;

    for child in GLOBAL_DIRS {
        let path = global_asset_dir(child)?;
        if !path.exists() {
            created_dirs.push(path.clone());
        }
        if path.starts_with(&dir) {
            root.create_dir_all(path.strip_prefix(&dir).expect("checked prefix"))?;
        } else {
            trusted_root(&path)?;
        }
    }

    let wrote_config = overwrite || !config_path.exists();
    if wrote_config {
        let encoded = toml::to_string_pretty(&config::default_config())?;
        root.write_atomic(
            config_path
                .strip_prefix(&dir)
                .expect("config is under global dir"),
            encoded,
        )?;
    }

    let context = RenderContext::new()
        .with("project", "project")
        .with("author", "Your Name")
        .with("year", "2026");
    for asset in assets::default_assets(&context) {
        let root = match asset.kind {
            assets::AssetKind::Template => "templates",
            assets::AssetKind::Profile => "profiles",
            assets::AssetKind::Snippet => "snippets",
            assets::AssetKind::Command => "commands",
            assets::AssetKind::Recipe => "recipes",
            assets::AssetKind::License => "licenses",
        };
        let asset_dir = global_asset_dir(root)?;
        let destination = asset_dir.join(&asset.path);
        if overwrite || !destination.exists() {
            let root = trusted_root(&asset_dir)?;
            if let Some(parent) = asset.path.parent() {
                root.create_dir_all(parent)?;
            }
            root.write_atomic(&asset.path, asset.contents)?;
        }
    }

    Ok(SetupReport {
        global_dir: dir,
        config_path,
        created_dirs,
        wrote_config,
    })
}

pub fn load_global_config() -> Result<config::LodeConfig> {
    ensure_global_workspace()?;

    let path = global_config_path()?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    let raw = migrate_config_source_if_needed(&path, &raw)?;
    let config: config::LodeConfig =
        toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
            path: PathBuf::from(path.as_str()),
            source,
        })?;
    config::validate_schema(&config)?;
    Ok(config)
}

pub fn save_global_config(config: &config::LodeConfig) -> Result<()> {
    config::validate_schema(config)?;
    ensure_global_workspace()?;
    let path = global_config_path()?;
    let encoded = toml::to_string_pretty(config)?;
    let parent = path
        .parent()
        .ok_or_else(|| LodeError::Message("global config path must have a parent".into()))?;
    trusted_root(parent)?.write_atomic(
        path.file_name()
            .ok_or_else(|| LodeError::Message("global config path must name a file".into()))?,
        encoded,
    )?;
    Ok(())
}

fn migrate_config_source_if_needed(path: &Utf8PathBuf, raw: &str) -> Result<String> {
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

fn trusted_root(path: impl AsRef<std::path::Path>) -> Result<ValidatedRoot> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|source| LodeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    ValidatedRoot::new(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::EnvGuard;

    #[test]
    fn global_workspace_path_respects_lode_config() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("custom").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config.to_str().unwrap());

        assert_eq!(
            global_dir().unwrap(),
            Utf8PathBuf::from_path_buf(config.parent().unwrap().to_path_buf()).unwrap()
        );
        assert_eq!(
            global_config_path().unwrap(),
            Utf8PathBuf::from_path_buf(config).unwrap()
        );
    }

    #[test]
    fn global_asset_dir_respects_specific_overrides() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("custom-templates");
        let _templates_guard = EnvGuard::set("LODE_TEMPLATES", templates.to_str().unwrap());

        assert_eq!(
            global_asset_dir("templates").unwrap(),
            Utf8PathBuf::from_path_buf(templates).unwrap()
        );
    }

    #[test]
    fn old_global_config_is_migrated_and_backed_up() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".lode").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config_path.to_str().unwrap());

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(
            &config_path,
            r#"
schema_version = 2
active_profile = "systems/rust-cli"

[identity]
author = "Ada"
"#,
        )
        .unwrap();

        let config = load_global_config().unwrap();

        assert_eq!(config.schema_version, config::SCHEMA_VERSION);
        assert_eq!(config.identity.author, "Ada");
        assert_eq!(config.identity.email, "you@example.com");
        assert_eq!(config.active_profile.as_deref(), Some("systems/rust-cli"));
        assert!(config_path
            .with_file_name("config.toml.bak-schema-2")
            .exists());
        assert!(fs::read_to_string(&config_path)
            .unwrap()
            .contains("schema_version = 3"));
    }

    #[test]
    fn future_global_config_schema_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".lode").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config_path.to_str().unwrap());

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        let mut config = config::default_config();
        config.schema_version = config::SCHEMA_VERSION + 1;
        fs::write(&config_path, toml::to_string_pretty(&config).unwrap()).unwrap();

        let error = load_global_config().unwrap_err();

        assert!(matches!(
            error,
            LodeError::SchemaMismatch {
                expected: config::SCHEMA_VERSION,
                found
            } if found == config::SCHEMA_VERSION + 1
        ));
    }
}
