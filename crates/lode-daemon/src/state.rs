use std::path::PathBuf;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub version: u32,
    pub state: DaemonState,
}

impl StateFile {
    pub fn new(state: DaemonState) -> Self {
        Self { version: 1, state }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub active: bool,
    pub paused: bool,
    pub watchers: Vec<String>,
    pub events_count: u64,
    pub started_at: Option<u64>,
}

impl Default for DaemonState {
    fn default() -> Self {
        Self {
            active: false,
            paused: false,
            watchers: Vec::new(),
            events_count: 0,
            started_at: None,
        }
    }
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

    pub fn remove_watcher(&mut self, name: &str) {
        self.watchers.retain(|w| w != name);
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

pub fn load_state(path: &PathBuf) -> Result<DaemonState, StateError> {
    if !path.exists() {
        return Ok(DaemonState::default());
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| StateError::FileError(e.to_string()))?;

    let state_file: StateFile =
        serde_json::from_str(&content).map_err(|e| StateError::SerializeError(e.to_string()))?;

    Ok(state_file.state)
}

pub fn save_state(path: &PathBuf, state: &DaemonState) -> Result<(), StateError> {
    let state_file = StateFile::new(state.clone());

    let content = serde_json::to_string_pretty(&state_file)
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
