use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::{LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateSyncReport {
    pub checked: usize,
    pub reconciled: usize,
    pub skipped: usize,
    pub planned_paths: Vec<Utf8PathBuf>,
    pub wrote_paths: Vec<Utf8PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncConfig {
    pub template_dirs: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            template_dirs: vec![
                ".lode/templates".to_string(),
                "_ref_".to_string(),
                "_ctx_".to_string(),
            ],
            exclude_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaleEntry {
    pub path: Utf8PathBuf,
    pub reason: StaleReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleReason {
    MissingSource,
    ModifiedSinceSync,
    OrphanedOutput,
}

pub fn detect_stale(project_dir: &std::path::Path) -> Result<Vec<StaleEntry>> {
    let lode_dir = project_dir.join(".lode");
    if !lode_dir.exists() {
        return Ok(Vec::new());
    }

    let mut stale = Vec::new();
    let lock_path = lode_dir.join("scaffold.lock");
    if lock_path.exists() {
        let raw = fs::read_to_string(&lock_path).map_err(|source| LodeError::Io {
            path: lock_path.clone(),
            source,
        })?;
        let lock: crate::ScaffoldLock =
            toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
                path: lock_path,
                source: Box::new(source),
            })?;
        for entry in &lock.entries {
            let dest = project_dir.join(&entry.destination);
            if !dest.exists() {
                stale.push(StaleEntry {
                    path: entry.destination.clone(),
                    reason: StaleReason::MissingSource,
                });
            }
        }
    }

    let sync_state_path = lode_dir.join("sync_state.json");
    if sync_state_path.exists() {
        let raw = fs::read_to_string(&sync_state_path).map_err(|source| LodeError::Io {
            path: sync_state_path.clone(),
            source,
        })?;
        let state: SyncState =
            serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
        for (rel_path, recorded_hash) in &state.content_hashes {
            let abs_path = project_dir.join(rel_path);
            if abs_path.exists() {
                let contents = fs::read_to_string(&abs_path).map_err(|source| LodeError::Io {
                    path: abs_path.clone(),
                    source,
                })?;
                let current_hash = content_hash(&contents);
                if &current_hash != recorded_hash {
                    stale.push(StaleEntry {
                        path: rel_path.clone(),
                        reason: StaleReason::ModifiedSinceSync,
                    });
                }
            } else {
                stale.push(StaleEntry {
                    path: rel_path.clone(),
                    reason: StaleReason::OrphanedOutput,
                });
            }
        }
    }

    Ok(stale)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncState {
    content_hashes: std::collections::BTreeMap<Utf8PathBuf, String>,
}

pub fn reconcile(
    project_dir: &std::path::Path,
    stale_paths: &[StaleEntry],
    dry_run: bool,
) -> Result<TemplateSyncReport> {
    let root = ValidatedRoot::new(project_dir)?;
    let mut planned = Vec::new();
    let mut wrote = Vec::new();
    let mut reconciled = 0usize;
    let mut skipped = 0usize;

    for entry in stale_paths {
        planned.push(entry.path.clone());
        match entry.reason {
            StaleReason::MissingSource => {
                skipped += 1;
            }
            StaleReason::ModifiedSinceSync | StaleReason::OrphanedOutput => {
                if dry_run {
                    skipped += 1;
                    continue;
                }
                if let Some(parent) = entry.path.parent() {
                    root.create_dir_all(parent)?;
                }
                let contents = "<!-- reconciled by lode -->\n".to_string();
                root.write_atomic(&entry.path, &contents)?;
                wrote.push(entry.path.clone());
                reconciled += 1;
            }
        }
    }

    Ok(TemplateSyncReport {
        checked: stale_paths.len(),
        reconciled,
        skipped,
        planned_paths: planned,
        wrote_paths: wrote,
    })
}

pub fn sync_templates(
    project_dir: &std::path::Path,
    config: &SyncConfig,
    force: bool,
    dry_run: bool,
) -> Result<TemplateSyncReport> {
    let mut planned = Vec::new();
    let mut wrote = Vec::new();
    let mut reconciled = 0usize;
    let mut skipped = 0usize;

    for dir_name in &config.template_dirs {
        let template_dir = project_dir.join(dir_name);
        if !template_dir.exists() {
            continue;
        }
        collect_template_targets(
            &template_dir,
            project_dir,
            &config.exclude_patterns,
            &mut planned,
        )?;
    }

    let checked = planned.len();

    if dry_run {
        return Ok(TemplateSyncReport {
            checked,
            reconciled: 0,
            skipped: checked,
            planned_paths: planned,
            wrote_paths: Vec::new(),
        });
    }

    let root = ValidatedRoot::new(project_dir)?;
    for rel_path in &planned {
        let abs_path = root.resolve(rel_path)?;
        if abs_path.exists() && !force {
            skipped += 1;
            continue;
        }
        if let Some(parent) = rel_path.parent() {
            root.create_dir_all(parent)?;
        }
        let contents = fs::read_to_string(&abs_path).unwrap_or_default();
        if !force && !contents.is_empty() {
            skipped += 1;
            continue;
        }
        root.write_atomic(rel_path, &contents)?;
        wrote.push(rel_path.clone());
        reconciled += 1;
    }

    Ok(TemplateSyncReport {
        checked,
        reconciled,
        skipped,
        planned_paths: planned,
        wrote_paths: wrote,
    })
}

fn collect_template_targets(
    dir: &std::path::Path,
    project_dir: &std::path::Path,
    exclude: &[String],
    out: &mut Vec<Utf8PathBuf>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| LodeError::Io {
            path: entry.path(),
            source,
        })?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if exclude.iter().any(|pattern| pattern == name_str.as_ref()) {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_template_targets(&path, project_dir, exclude, out)?;
        } else {
            let relative = path.strip_prefix(project_dir).unwrap_or(&path);
            if let Ok(utf8) = Utf8PathBuf::from_path_buf(relative.to_path_buf()) {
                out.push(utf8);
            }
        }
    }
    Ok(())
}

fn content_hash(contents: &str) -> String {
    crate::signature::compute_content_hash(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_project() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().to_path_buf();
        (temp, dir)
    }

    #[test]
    fn detect_stale_returns_empty_without_lode_dir() {
        let (_temp, dir) = temp_project();
        let stale = detect_stale(&dir).unwrap();
        assert!(stale.is_empty());
    }

    #[test]
    fn detect_stale_finds_missing_lock_entries() {
        let (_temp, dir) = temp_project();
        let lode_dir = dir.join(".lode");
        fs::create_dir_all(&lode_dir).unwrap();
        let lock = crate::ScaffoldLock {
            schema_version: 3,
            generated_by: "lode".to_string(),
            project: "test".to_string(),
            entries: vec![crate::ScaffoldLockEntry {
                template: "root/README.md".to_string(),
                destination: Utf8PathBuf::from("README.md"),
                content_hash: "abc".to_string(),
            }],
        };
        fs::write(
            lode_dir.join("scaffold.lock"),
            toml::to_string_pretty(&lock).unwrap(),
        )
        .unwrap();

        let stale = detect_stale(&dir).unwrap();
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].reason, StaleReason::MissingSource);
    }

    #[test]
    fn sync_templates_dry_run_writes_nothing() {
        let (_temp, dir) = temp_project();
        let config = SyncConfig::default();
        let report = sync_templates(&dir, &config, false, true).unwrap();
        assert!(report.wrote_paths.is_empty());
    }

    #[test]
    fn reconcile_dry_run_writes_nothing() {
        let (_temp, dir) = temp_project();
        let stale = vec![StaleEntry {
            path: Utf8PathBuf::from("stale.txt"),
            reason: StaleReason::ModifiedSinceSync,
        }];
        let report = reconcile(&dir, &stale, true).unwrap();
        assert_eq!(report.reconciled, 0);
        assert!(!dir.join("stale.txt").exists());
    }

    #[test]
    fn reconcile_writes_missing_files() {
        let (_temp, dir) = temp_project();
        let stale = vec![StaleEntry {
            path: Utf8PathBuf::from("recovered.txt"),
            reason: StaleReason::ModifiedSinceSync,
        }];
        let report = reconcile(&dir, &stale, false).unwrap();
        assert_eq!(report.reconciled, 1);
        assert!(dir.join("recovered.txt").exists());
    }

    #[test]
    fn reconcile_rejects_path_traversal() {
        let (_temp, dir) = temp_project();
        let stale = vec![StaleEntry {
            path: Utf8PathBuf::from("../escape.txt"),
            reason: StaleReason::ModifiedSinceSync,
        }];
        assert!(reconcile(&dir, &stale, false).is_err());
        assert!(!dir.parent().unwrap().join("escape.txt").exists());
    }

    #[test]
    fn default_config_has_expected_dirs() {
        let config = SyncConfig::default();
        assert!(config
            .template_dirs
            .contains(&".lode/templates".to_string()));
        assert!(config.exclude_patterns.contains(&".git".to_string()));
    }
}
