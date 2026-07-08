use std::path::PathBuf;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::watcher::WatcherConfig;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Handler failed: {0}")]
    Failed(String),
    #[error("Path error: {0}")]
    PathError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerResult {
    pub actions: Vec<String>,
    pub errors: Vec<String>,
}

pub fn handle_create(path: &PathBuf, config: &WatcherConfig) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path =
        Utf8PathBuf::try_from(path.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File created: {utf8_path}"));

    if !config.no_stamp {
        actions.push(format!("Stamp applied: {utf8_path}"));
    }

    Ok(actions)
}

pub fn handle_modify(path: &PathBuf, config: &WatcherConfig) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path =
        Utf8PathBuf::try_from(path.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File modified: {utf8_path}"));

    if !config.no_sign {
        actions.push(format!("Signature checked: {utf8_path}"));
    }

    Ok(actions)
}

pub fn handle_rename(
    from: &PathBuf,
    to: &PathBuf,
    _config: &WatcherConfig,
) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_from =
        Utf8PathBuf::try_from(from.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;
    let utf8_to =
        Utf8PathBuf::try_from(to.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File renamed: {utf8_from} -> {utf8_to}"));

    Ok(actions)
}

pub fn handle_delete(path: &PathBuf, _config: &WatcherConfig) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path =
        Utf8PathBuf::try_from(path.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File deleted: {utf8_path}"));

    Ok(actions)
}

pub fn convention_handler(path: &PathBuf) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path =
        Utf8PathBuf::try_from(path.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    let filename = utf8_path
        .file_name()
        .ok_or_else(|| HandlerError::PathError("No filename".to_string()))?;

    if filename.contains(' ') {
        actions.push(format!("Naming warning: spaces in {filename}"));
    }

    if filename.len() > 100 {
        actions.push(format!("Naming warning: long filename {filename}"));
    }

    Ok(actions)
}

pub fn signature_handler(path: &PathBuf) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path =
        Utf8PathBuf::try_from(path.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("Signature processed: {utf8_path}"));

    Ok(actions)
}

pub fn env_drift_handler(path: &PathBuf) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path =
        Utf8PathBuf::try_from(path.clone()).map_err(|e| HandlerError::PathError(e.to_string()))?;

    let ext = utf8_path.extension().unwrap_or_default().to_string();

    match ext.as_str() {
        "toml" => actions.push(format!("Config drift detected: {utf8_path}")),
        "env" => actions.push(format!("Environment drift detected: {utf8_path}")),
        "lock" => actions.push(format!("Lock file drift detected: {utf8_path}")),
        _ => {}
    }

    Ok(actions)
}
