use std::path::Path;
use std::time::SystemTime;

use lode_core::ValidatedRoot;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("State file error: {0}")]
    FileError(String),
    #[error("Serialization error: {0}")]
    SerializeError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaemonState {
    pub active: bool,
    pub paused: bool,
    pub watchers: Vec<String>,
    pub events_count: u64,
    pub started_at: Option<u64>,
}

impl DaemonState {
    pub fn new() -> Self {
        let started_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());

        Self {
            active: true,
            paused: false,
            watchers: Vec::new(),
            events_count: 0,
            started_at,
        }
    }

    pub fn increment_events(&mut self) {
        self.events_count += 1;
    }

    pub fn add_watcher(&mut self, name: String) {
        if !self.watchers.contains(&name) {
            self.watchers.push(name);
        }
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

pub fn load_state(path: &Path) -> Result<DaemonState, StateError> {
    if !path.exists() {
        return Ok(DaemonState::default());
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| StateError::FileError(e.to_string()))?;

    let state: DaemonState =
        serde_json::from_str(&content).map_err(|e| StateError::SerializeError(e.to_string()))?;

    Ok(state)
}

pub fn save_state(path: &Path, state: &DaemonState) -> Result<(), StateError> {
    let content = serde_json::to_string_pretty(&state)
        .map_err(|e| StateError::SerializeError(e.to_string()))?;

    let parent = path
        .parent()
        .ok_or_else(|| StateError::FileError("state path has no parent".to_string()))?;
    if !parent.exists() {
        let base = parent
            .parent()
            .ok_or_else(|| StateError::FileError("state directory has no parent".to_string()))?;
        let dir_name = parent
            .file_name()
            .ok_or_else(|| StateError::FileError("state directory has no name".to_string()))?;
        ValidatedRoot::new(base)
            .map_err(|e| StateError::FileError(e.to_string()))?
            .create_dir_all(dir_name)
            .map_err(|e| StateError::FileError(e.to_string()))?;
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| StateError::FileError("state file has no name".to_string()))?;
    ValidatedRoot::new(parent)
        .map_err(|e| StateError::FileError(e.to_string()))?
        .write_atomic(file_name, content)
        .map_err(|e| StateError::FileError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod daemon_state_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn new_state_is_active_and_not_paused() {
        let state = DaemonState::new();
        assert!(state.active);
        assert!(!state.paused);
        assert_eq!(state.events_count, 0);
        assert!(state.watchers.is_empty());
    }

    #[test]
    fn increment_events_increases_count() {
        let mut state = DaemonState::new();
        state.increment_events();
        assert_eq!(state.events_count, 1);
        state.increment_events();
        assert_eq!(state.events_count, 2);
    }

    #[test]
    fn add_watcher_does_not_duplicate() {
        let mut state = DaemonState::new();
        state.add_watcher("src".to_string());
        state.add_watcher("src".to_string());
        assert_eq!(state.watchers.len(), 1);
    }

    #[test]
    fn stop_deactivates() {
        let mut state = DaemonState::new();
        state.stop();
        assert!(!state.active);
    }

    #[test]
    fn load_state_returns_default_for_missing_file() {
        let state = load_state(Path::new("nonexistent.json")).unwrap();
        assert!(!state.active);
        assert_eq!(state.events_count, 0);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("state.json");

        let mut state = DaemonState::new();
        state.add_watcher("src".to_string());
        state.increment_events();

        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();

        assert_eq!(loaded.active, state.active);
        assert_eq!(loaded.paused, state.paused);
        assert_eq!(loaded.events_count, state.events_count);
        assert_eq!(loaded.watchers, state.watchers);
    }

    #[test]
    fn load_state_errors_on_invalid_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, b"not json").unwrap();
        assert!(load_state(&path).is_err());
    }

    #[test]
    fn save_state_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sub").join("state.json");
        let state = DaemonState::new();
        save_state(&path, &state).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn default_state_is_inactive() {
        let state = DaemonState::default();
        assert!(!state.active);
    }
}
