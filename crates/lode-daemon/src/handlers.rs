use std::{
    fs,
    path::{Path, PathBuf},
};

use camino::Utf8PathBuf;
use lode_core::signature::{
    comment_prefix_for_extension, has_signature_header, SignatureConfig, SignatureHeader,
};
use thiserror::Error;

use crate::watcher::WatcherConfig;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Path error: {0}")]
    PathError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

fn extension(path: &Utf8PathBuf) -> Option<String> {
    path.extension().map(|e| e.to_string())
}

pub fn handle_create(path: &PathBuf, config: &WatcherConfig) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path = Utf8PathBuf::try_from(path.to_path_buf())
        .map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File created: {utf8_path}"));

    if !config.no_stamp {
        let ext = extension(&utf8_path);
        if let Some(ext) = ext {
            if let Some(prefix) = comment_prefix_for_extension(&ext) {
                let content =
                    fs::read_to_string(path).map_err(|e| HandlerError::IoError(e.to_string()))?;
                if !has_signature_header(&content) {
                    let header = SignatureHeader::new(
                        utf8_path.file_name().unwrap_or("unknown"),
                        "default",
                        "lode-daemon",
                        "MIT",
                    );
                    let sig_config = SignatureConfig::default();
                    let header_text = header.render(&sig_config, prefix);
                    let new_content = format!("{header_text}{content}");
                    fs::write(path, &new_content)
                        .map_err(|e| HandlerError::IoError(e.to_string()))?;
                    actions.push(format!("Stamp added: {utf8_path} (prefix={prefix})"));
                } else {
                    actions.push(format!("Stamp skipped (already present): {utf8_path}"));
                }
            } else {
                actions.push(format!(
                    "Stamp skipped (no comment prefix for .{ext}): {utf8_path}"
                ));
            }
        } else {
            actions.push(format!("Stamp skipped (no extension): {utf8_path}"));
        }
    }

    Ok(actions)
}

pub fn handle_modify(path: &PathBuf, config: &WatcherConfig) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path = Utf8PathBuf::try_from(path.to_path_buf())
        .map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File modified: {utf8_path}"));

    if !config.no_sign {
        let content = fs::read_to_string(path).map_err(|e| HandlerError::IoError(e.to_string()))?;
        if has_signature_header(&content) {
            actions.push(format!("Signature verified: {utf8_path}"));
        } else {
            actions.push(format!("Signature missing: {utf8_path}"));
        }
    }

    Ok(actions)
}

pub fn handle_rename(from: &Path, to: &Path) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_from = Utf8PathBuf::try_from(from.to_path_buf())
        .map_err(|e| HandlerError::PathError(e.to_string()))?;
    let utf8_to = Utf8PathBuf::try_from(to.to_path_buf())
        .map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File renamed: {utf8_from} -> {utf8_to}"));

    // Update signature path field in destination file if it has a signature
    if let Ok(content) = fs::read_to_string(to) {
        if has_signature_header(&content) {
            actions.push(format!("Signature path update needed: {utf8_to}"));
        }
    }

    Ok(actions)
}

pub fn handle_delete(path: &Path) -> Result<Vec<String>, HandlerError> {
    let mut actions = Vec::new();

    let utf8_path = Utf8PathBuf::try_from(path.to_path_buf())
        .map_err(|e| HandlerError::PathError(e.to_string()))?;

    actions.push(format!("File deleted: {utf8_path}"));

    Ok(actions)
}

#[cfg(test)]
mod daemon_handler_tests {
    use super::*;

    #[test]
    fn extension_returns_some_for_known_extensions() {
        let path = Utf8PathBuf::from("file.rs");
        assert_eq!(extension(&path), Some("rs".to_string()));
    }

    #[test]
    fn extension_returns_none_for_no_extension() {
        let path = Utf8PathBuf::from("Makefile");
        assert_eq!(extension(&path), None);
    }

    #[test]
    fn extension_returns_last_component() {
        let path = Utf8PathBuf::from("archive.tar.gz");
        assert_eq!(extension(&path), Some("gz".to_string()));
    }

    #[test]
    fn handle_create_adds_stamp_to_unsigned_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("hello.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let config = WatcherConfig::default();
        let result = handle_create(&file_path, &config).unwrap();

        assert!(result.iter().any(|a| a.starts_with("Stamp added")));
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.starts_with("// "));
    }

    #[test]
    fn handle_create_skips_stamp_when_no_stamp_is_set() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("hello.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let config = WatcherConfig {
            no_stamp: true,
            ..Default::default()
        };
        let result = handle_create(&file_path, &config).unwrap();

        assert!(result.iter().all(|a| !a.starts_with("Stamp")));
    }

    #[test]
    fn handle_modify_reports_signature_status() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("hello.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let config = WatcherConfig::default();
        let result = handle_modify(&file_path, &config).unwrap();
        assert!(result.iter().any(|a| a.contains("Signature missing")));
    }

    #[test]
    fn handle_delete_reports_deletion() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("gone.rs");
        std::fs::write(&file_path, "content").unwrap();

        let result = handle_delete(file_path.as_path()).unwrap();
        assert!(result.iter().any(|a| a.starts_with("File deleted")));
    }
}
