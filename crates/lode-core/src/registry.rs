use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{global_dir, LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    pub projects: Vec<ProjectRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub name: String,
    pub path: Utf8PathBuf,
    pub profile: String,
    pub last_seen: String,
}

pub fn registry_path() -> Result<Utf8PathBuf> {
    Ok(global_dir()?.join("registry.json"))
}

pub fn load_registry() -> Result<Registry> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(Registry::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub fn save_registry(registry: &Registry) -> Result<()> {
    let global = global_dir()?;
    let parent = global
        .parent()
        .ok_or_else(|| LodeError::Message("global directory has no parent".to_string()))?;
    let relative = global
        .file_name()
        .ok_or_else(|| LodeError::Message("global directory has no file name".to_string()))?;
    ValidatedRoot::new(parent)?.create_dir_all(relative)?;
    let root = ValidatedRoot::new(&global)?;
    let raw = serde_json::to_string_pretty(registry)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic("registry.json", raw)?;
    Ok(())
}

pub fn register_project(name: &str, path: &Utf8Path, profile: &str) -> Result<ProjectRecord> {
    let mut registry = load_registry()?;
    let record = ProjectRecord {
        name: name.to_string(),
        path: path.to_path_buf(),
        profile: profile.to_string(),
        last_seen: now_stamp(),
    };
    registry
        .projects
        .retain(|existing| existing.path != record.path);
    registry.projects.push(record.clone());
    registry
        .projects
        .sort_by(|left, right| left.name.cmp(&right.name));
    save_registry(&registry)?;
    Ok(record)
}

pub fn prune_registry() -> Result<usize> {
    let mut registry = load_registry()?;
    let before = registry.projects.len();
    registry.projects.retain(|record| record.path.exists());
    let removed = before - registry.projects.len();
    save_registry(&registry)?;
    Ok(removed)
}

fn now_stamp() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::EnvGuard;

    #[test]
    fn registers_and_prunes_projects() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join(".lode").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config.to_str().unwrap());
        let project = Utf8PathBuf::from_path_buf(temp.path().join("app")).unwrap();
        fs::create_dir_all(&project).unwrap();

        register_project("app", &project, "core/app").unwrap();
        assert_eq!(load_registry().unwrap().projects.len(), 1);

        fs::remove_dir_all(&project).unwrap();
        assert_eq!(prune_registry().unwrap(), 1);
    }
}
