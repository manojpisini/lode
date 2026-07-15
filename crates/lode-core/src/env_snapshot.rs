use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSnapshot {
    pub id: String,
    pub label: String,
    pub created_at: u64,
    pub variables: HashMap<String, String>,
}

fn snapshots_path() -> Result<std::path::PathBuf> {
    let dir = crate::install::global_asset_dir("state")?;
    Ok(dir.join("env-snapshots.json").into())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs()
}

pub fn list_snapshots() -> Result<Vec<EnvSnapshot>> {
    let path = snapshots_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    let snapshots: Vec<EnvSnapshot> =
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))?;
    Ok(snapshots)
}

fn is_sensitive_env_var(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "secret",
        "token",
        "password",
        "private_key",
        "key",
        "access_key",
        "secret_key",
        "auth",
        "credential",
    ]
    .iter()
    .any(|&kw| lower.contains(kw))
}

pub fn create_snapshot(label: &str) -> Result<EnvSnapshot> {
    let vars: HashMap<String, String> = std::env::vars()
        .filter(|(k, _)| !is_sensitive_env_var(k))
        .collect();
    let snapshot = EnvSnapshot {
        id: format!("snap-{:x}", now_secs()),
        label: label.to_string(),
        created_at: now_secs(),
        variables: vars,
    };
    let mut snapshots = list_snapshots()?;
    snapshots.push(snapshot.clone());
    save_snapshots(&snapshots)?;
    Ok(snapshot)
}

pub fn compare_snapshots(id1: &str, id2: &str) -> Result<EnvDiff> {
    let snapshots = list_snapshots()?;
    let s1 = snapshots
        .iter()
        .find(|s| s.id == id1)
        .ok_or_else(|| LodeError::Message(format!("snapshot not found: {id1}")))?;
    let s2 = snapshots
        .iter()
        .find(|s| s.id == id2)
        .ok_or_else(|| LodeError::Message(format!("snapshot not found: {id2}")))?;
    Ok(compute_diff(s1, s2))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<(String, String, String)>,
    pub same: usize,
}

fn compute_diff(a: &EnvSnapshot, b: &EnvSnapshot) -> EnvDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut same = 0usize;

    for (k, v) in &a.variables {
        match b.variables.get(k) {
            None => removed.push(k.clone()),
            Some(bv) if bv == v => same += 1,
            Some(bv) => changed.push((k.clone(), v.clone(), bv.clone())),
        }
    }
    for (k, _) in &b.variables {
        if !a.variables.contains_key(k) {
            added.push(k.clone());
        }
    }

    EnvDiff {
        added,
        removed,
        changed,
        same,
    }
}

pub fn restore_snapshot(id: &str) -> Result<()> {
    let snapshots = list_snapshots()?;
    let snapshot = snapshots
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| LodeError::Message(format!("snapshot not found: {id}")))?;
    for (k, v) in &snapshot.variables {
        std::env::set_var(k, v);
    }
    Ok(())
}

fn save_snapshots(snapshots: &[EnvSnapshot]) -> Result<()> {
    let path = snapshots_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let raw =
        serde_json::to_string_pretty(snapshots).map_err(|e| LodeError::Message(e.to_string()))?;
    fs::write(&path, &raw).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snap(id: &str, vars: Vec<(&str, &str)>) -> EnvSnapshot {
        EnvSnapshot {
            id: id.to_string(),
            label: id.to_string(),
            created_at: 0,
            variables: vars
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn test_compute_diff_identical() {
        let a = make_snap("a", vec![("X", "1"), ("Y", "2")]);
        let b = make_snap("b", vec![("X", "1"), ("Y", "2")]);
        let d = compute_diff(&a, &b);
        assert_eq!(d.same, 2);
        assert!(d.added.is_empty());
        assert!(d.removed.is_empty());
    }

    #[test]
    fn test_compute_diff_added_removed_changed() {
        let a = make_snap("a", vec![("X", "1"), ("Y", "2")]);
        let b = make_snap("b", vec![("Y", "3"), ("Z", "4")]);
        let d = compute_diff(&a, &b);
        assert_eq!(d.removed, vec!["X"]);
        assert_eq!(d.added, vec!["Z"]);
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].0, "Y");
    }

    #[test]
    fn test_list_empty_when_no_file() {
        let result = list_snapshots();
        assert!(result.is_ok());
    }
}
