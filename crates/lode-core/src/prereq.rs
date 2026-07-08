use serde::{Deserialize, Serialize};

use crate::{error::Result, Process};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrereqConfig {
    pub check_on_init: bool,
    pub guided_install: bool,
    pub tools: Vec<PrereqTool>,
}

impl Default for PrereqConfig {
    fn default() -> Self {
        Self {
            check_on_init: true,
            guided_install: true,
            tools: vec![
                PrereqTool {
                    name: "git".to_string(),
                    command: "git".to_string(),
                    version_arg: "--version".to_string(),
                    required: true,
                    min_version: Some("2.30.0".to_string()),
                    install_hint: "https://git-scm.com/downloads".to_string(),
                },
                PrereqTool {
                    name: "rustup".to_string(),
                    command: "rustup".to_string(),
                    version_arg: "--version".to_string(),
                    required: false,
                    min_version: None,
                    install_hint: "https://rustup.rs".to_string(),
                },
                PrereqTool {
                    name: "node".to_string(),
                    command: "node".to_string(),
                    version_arg: "--version".to_string(),
                    required: false,
                    min_version: Some("20.0.0".to_string()),
                    install_hint: "https://nodejs.org".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrereqTool {
    pub name: String,
    pub command: String,
    pub version_arg: String,
    pub required: bool,
    pub min_version: Option<String>,
    pub install_hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrereqReport {
    pub checks: Vec<PrereqCheck>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrereqCheck {
    pub name: String,
    pub status: String,
    pub version: Option<String>,
    pub required: bool,
    pub install_hint: String,
}

pub fn check_prerequisites(config: &PrereqConfig) -> Result<PrereqReport> {
    let mut checks = Vec::new();
    for tool in &config.tools {
        let installed_version = detect_tool_version(&tool.command, &tool.version_arg);
        let status = match &installed_version {
            Some(version) => {
                if let Some(ref min) = tool.min_version {
                    if version_meets_minimum(version, min) {
                        "ok".to_string()
                    } else {
                        "warn".to_string()
                    }
                } else {
                    "ok".to_string()
                }
            }
            None => {
                if tool.required {
                    "fail".to_string()
                } else {
                    "warn".to_string()
                }
            }
        };
        checks.push(PrereqCheck {
            name: tool.name.clone(),
            status,
            version: installed_version,
            required: tool.required,
            install_hint: tool.install_hint.clone(),
        });
    }
    Ok(PrereqReport { checks })
}

pub fn detect_tool_version(command: &str, version_arg: &str) -> Option<String> {
    let output = Process::new(command)
        .ok()?
        .args([version_arg])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_version(&stdout)
}

fn extract_version(output: &str) -> Option<String> {
    let re = regex_simple(r"v?(\d+\.\d+(?:\.\d+)?)", output)?;
    Some(re)
}

fn regex_simple(_pattern: &str, text: &str) -> Option<String> {
    let _pattern = _pattern
        .replace(r"v?", "")
        .replace(r"\.", ".")
        .replace(r"(?:\.\d+)?", "");
    let words: Vec<&str> = text.split_whitespace().collect();
    for word in words {
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

fn version_meets_minimum(actual: &str, minimum: &str) -> bool {
    let actual_parts: Vec<u32> = actual.split('.').filter_map(|p| p.parse().ok()).collect();
    let min_parts: Vec<u32> = minimum.split('.').filter_map(|p| p.parse().ok()).collect();
    for (a, m) in actual_parts.iter().zip(min_parts.iter()) {
        if a > m {
            return true;
        }
        if a < m {
            return false;
        }
    }
    actual_parts.len() >= min_parts.len()
}

pub fn required_tools_for_project() -> Vec<String> {
    vec!["git".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(version_meets_minimum("2.43.0", "2.30.0"));
        assert!(version_meets_minimum("20.0.0", "20.0.0"));
        assert!(!version_meets_minimum("1.9.0", "2.0.0"));
    }
}
