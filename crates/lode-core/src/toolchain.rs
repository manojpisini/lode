use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    error::{LodeError, Result},
    Process, ValidatedRoot,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainConfig {
    pub auto_detect: bool,
    pub runtimes: Vec<RuntimeConfig>,
}

impl Default for ToolchainConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            runtimes: vec![
                RuntimeConfig {
                    name: "node".to_string(),
                    manager: "fnm".to_string(),
                    lock_file: ".nvmrc".to_string(),
                    version_cmd: "node --version".to_string(),
                },
                RuntimeConfig {
                    name: "rust".to_string(),
                    manager: "rustup".to_string(),
                    lock_file: "rust-toolchain.toml".to_string(),
                    version_cmd: "rustc --version".to_string(),
                },
                RuntimeConfig {
                    name: "python".to_string(),
                    manager: "pyenv".to_string(),
                    lock_file: ".python-version".to_string(),
                    version_cmd: "python --version".to_string(),
                },
                RuntimeConfig {
                    name: "go".to_string(),
                    manager: "go".to_string(),
                    lock_file: "go.mod".to_string(),
                    version_cmd: "go version".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub name: String,
    pub manager: String,
    pub lock_file: String,
    pub version_cmd: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolchainStore {
    pub runtimes: BTreeMap<String, Vec<String>>,
    pub active: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainStatus {
    pub runtime: String,
    pub installed: bool,
    pub version: Option<String>,
    pub lock_version: Option<String>,
    pub manager: String,
}

pub fn detect_installed_version(version_cmd: &str) -> Option<String> {
    let parts: Vec<&str> = version_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    // Reject parts containing shell metacharacters
    let dangerous = [
        ';', '|', '&', '`', '$', '(', ')', '{', '}', '<', '>', '\\', '\'', '"', '!', '#',
    ];
    if parts
        .iter()
        .any(|p| p.chars().any(|c| dangerous.contains(&c)))
    {
        return None;
    }
    let output = Process::new(parts[0])
        .ok()?
        .args(&parts[1..])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    extract_version_from_output(&combined)
}

fn extract_version_from_output(output: &str) -> Option<String> {
    for word in output.split_whitespace() {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.len() >= 2 {
            if let (Ok(maj), Ok(min)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if maj > 0 || min > 0 {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

pub fn read_lock_version(project_dir: &Path, lock_file: &str) -> Option<String> {
    if lock_file.ends_with(".toml") {
        let path = project_dir.join(lock_file);
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("channel") || trimmed.starts_with("version") {
                if let Some(value) = trimmed.split_once('=') {
                    let val = value.1.trim().trim_matches('"').trim_matches('\'');
                    return Some(val.to_string());
                }
            }
        }
    } else {
        let path = project_dir.join(lock_file);
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        let first_line = content.lines().next()?.trim();
        if !first_line.is_empty() {
            return Some(first_line.to_string());
        }
    }
    None
}

pub fn toolchain_status(project_dir: &Path, config: &ToolchainConfig) -> Vec<ToolchainStatus> {
    let mut statuses = Vec::new();
    for runtime in &config.runtimes {
        let installed_version = detect_installed_version(&runtime.version_cmd);
        let lock_version = read_lock_version(project_dir, &runtime.lock_file);
        statuses.push(ToolchainStatus {
            runtime: runtime.name.clone(),
            installed: installed_version.is_some(),
            version: installed_version,
            lock_version,
            manager: runtime.manager.clone(),
        });
    }
    statuses
}

pub fn pin_runtime(project_dir: &Path, runtime: &RuntimeConfig, version: &str) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    match runtime.name.as_str() {
        "rust" => {
            let content = format!("[toolchain]\nchannel = \"{version}\"\n");
            root.write_atomic(&runtime.lock_file, content)?;
        }
        "node" | "typescript" | "python" => {
            root.write_atomic(&runtime.lock_file, format!("{version}\n"))?;
        }
        _ => {
            root.write_atomic(&runtime.lock_file, format!("{version}\n"))?;
        }
    }
    Ok(())
}

pub fn load_store(project_dir: &Path) -> ToolchainStore {
    let path = project_dir.join(".lode").join("toolchain.json");
    if path.exists() {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_e) => {
                let backup = path.with_extension("json.bak");
                fs::rename(&path, &backup).ok();
                eprintln!(
                    "lode: warning: corrupted toolchain data backed up to {:?}",
                    backup
                );
                String::new()
            }
        };
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ToolchainStore::default()
    }
}

pub fn save_store(project_dir: &Path, store: &ToolchainStore) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    root.create_dir_all(".lode")?;
    let content =
        serde_json::to_string_pretty(store).map_err(|e| LodeError::Message(e.to_string()))?;
    root.write_atomic(".lode/toolchain.json", content)
        .map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_version() {
        assert_eq!(
            extract_version_from_output("v22.4.0"),
            Some("22.4.0".to_string())
        );
        assert_eq!(
            extract_version_from_output("node v20.11.1"),
            Some("20.11.1".to_string())
        );
    }
}
