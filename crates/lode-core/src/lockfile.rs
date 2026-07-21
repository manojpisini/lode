use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LodeError, Result};

const LOCKFILE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LodeLock {
    pub schema_version: u32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub assets: Vec<LockAssetEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockAssetEntry {
    pub id: String,
    pub version: String,
    pub sha256: String,
    pub kind: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockDiff {
    pub added: Vec<LockAssetEntry>,
    pub removed: Vec<LockAssetEntry>,
    pub changed: Vec<(LockAssetEntry, LockAssetEntry)>,
    pub unchanged: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockVerifyReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub checked: usize,
}

pub fn lockfile_path(project_dir: &Utf8PathBuf) -> Utf8PathBuf {
    project_dir.join(".lode").join("lode.lock")
}

pub fn load_lock(path: &Utf8PathBuf) -> Result<LodeLock> {
    let raw = fs::read_to_string(path.as_std_path()).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let lock: LodeLock = toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
        path: PathBuf::from(path.as_str()),
        source: Box::new(source),
    })?;
    if lock.schema_version != LOCKFILE_SCHEMA_VERSION {
        return Err(LodeError::SchemaMismatch {
            expected: LOCKFILE_SCHEMA_VERSION,
            found: lock.schema_version,
        });
    }
    Ok(lock)
}

pub fn save_lock(path: &Utf8PathBuf, lock: &LodeLock) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent.as_std_path()).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw =
        toml::to_string_pretty(lock).map_err(|source| LodeError::Message(source.to_string()))?;
    fs::write(path.as_std_path(), &raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(())
}

pub fn new_lock() -> LodeLock {
    let ts = timestamp();
    LodeLock {
        schema_version: LOCKFILE_SCHEMA_VERSION,
        created_at: ts.clone(),
        updated_at: ts,
        assets: Vec::new(),
    }
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{secs}")
}

pub fn hash_file(path: &Utf8PathBuf) -> Result<String> {
    let data = fs::read(path.as_std_path()).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(crate::util::hex_lower(hasher.finalize()))
}

pub fn verify_lock(lock: &LodeLock) -> LockVerifyReport {
    verify_lock_in(lock, None)
}

pub fn verify_lock_in(lock: &LodeLock, root: Option<&std::path::Path>) -> LockVerifyReport {
    let root = root.and_then(|r| std::path::Path::new(r).canonicalize().ok());
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut checked = 0;

    for entry in &lock.assets {
        checked += 1;
        if let Some(ref path) = entry.path {
            let p = Utf8PathBuf::from(path);
            if let Some(ref root) = root {
                let canonical = std::path::Path::new(&p).canonicalize();
                match canonical {
                    Ok(abs_path) => {
                        if !abs_path.starts_with(root) {
                            errors.push(format!("{}: path outside project root: {path}", entry.id));
                            continue;
                        }
                    }
                    Err(_) => {
                        errors.push(format!("{}: cannot resolve path: {path}", entry.id));
                        continue;
                    }
                }
            }
            if !p.exists() {
                errors.push(format!("{}: file not found: {path}", entry.id));
                continue;
            }
            match hash_file(&p) {
                Ok(hash) => {
                    if hash != entry.sha256 {
                        errors.push(format!(
                            "{}: hash mismatch (expected {}, got {}) at {path}",
                            entry.id, entry.sha256, hash
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!("{}: failed to hash {path}: {e}", entry.id));
                }
            }
        } else {
            warnings.push(format!(
                "{}: no path recorded, skipping hash verification",
                entry.id
            ));
        }
    }

    let valid = errors.is_empty();
    LockVerifyReport {
        valid,
        errors,
        warnings,
        checked,
    }
}

pub fn diff_locks(current: &LodeLock, expected: &LodeLock) -> LockDiff {
    let mut current_map: HashMap<&str, &LockAssetEntry> = HashMap::new();
    for entry in &current.assets {
        current_map.insert(&entry.id, entry);
    }

    let mut expected_map: HashMap<&str, &LockAssetEntry> = HashMap::new();
    for entry in &expected.assets {
        expected_map.insert(&entry.id, entry);
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut unchanged = 0;

    for (id, entry) in &expected_map {
        match current_map.get(id) {
            None => added.push((*entry).clone()),
            Some(cur) => {
                if cur.version != entry.version || cur.sha256 != entry.sha256 {
                    changed.push(((*cur).clone(), (*entry).clone()));
                } else {
                    unchanged += 1;
                }
            }
        }
    }

    for (id, entry) in &current_map {
        if !expected_map.contains_key(id) {
            removed.push((*entry).clone());
        }
    }

    LockDiff {
        added,
        removed,
        changed,
        unchanged,
    }
}

pub fn update_lock(lock: &mut LodeLock, assets: Vec<LockAssetEntry>) -> LockDiff {
    let old = LodeLock {
        schema_version: lock.schema_version,
        created_at: lock.created_at.clone(),
        updated_at: lock.updated_at.clone(),
        assets: lock.assets.clone(),
    };

    // Replace assets with matching IDs, append new ones
    let mut new_assets: Vec<LockAssetEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // First pass: keep existing assets that aren't being replaced
    for existing in &lock.assets {
        if !assets.iter().any(|a| a.id == existing.id) {
            new_assets.push(existing.clone());
            seen.insert(existing.id.clone());
        }
    }

    // Second pass: add/update from new assets
    for asset in assets {
        new_assets.push(asset);
    }

    lock.assets = new_assets;
    lock.updated_at = timestamp();

    diff_locks(&old, lock)
}
