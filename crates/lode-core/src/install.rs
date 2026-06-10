use std::{env, fs, path::PathBuf};

use camino::Utf8PathBuf;

use crate::{assets, config, template::RenderContext, LodeError, Result};

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

pub fn ensure_global_workspace() -> Result<()> {
    let dir = global_dir()?;
    create_dir_all(&dir)?;

    for child in GLOBAL_DIRS {
        create_dir_all(&dir.join(child))?;
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
    create_dir_all(&dir)?;

    for child in GLOBAL_DIRS {
        let path = dir.join(child);
        if !path.exists() {
            created_dirs.push(path.clone());
        }
        create_dir_all(&path)?;
    }

    let wrote_config = overwrite || !config_path.exists();
    if wrote_config {
        let encoded = toml::to_string_pretty(&config::default_config())?;
        fs::write(&config_path, encoded).map_err(|source| LodeError::Io {
            path: PathBuf::from(config_path.as_str()),
            source,
        })?;
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
        let destination = dir.join(root).join(asset.path);
        if overwrite || !destination.exists() {
            if let Some(parent) = destination.parent() {
                create_dir_all(&parent.to_path_buf())?;
            }
            fs::write(&destination, asset.contents).map_err(|source| LodeError::Io {
                path: PathBuf::from(destination.as_str()),
                source,
            })?;
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
    fs::write(&path, encoded).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })
}

fn create_dir_all(path: &Utf8PathBuf) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })
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
}
