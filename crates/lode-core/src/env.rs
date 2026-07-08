use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    error::{LodeError, Result},
    ValidatedRoot,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub auto_create: bool,
    pub runtime_lock: bool,
    pub vars: Vec<EnvVar>,
    #[serde(default)]
    pub validation: EnvValidation,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            auto_create: true,
            runtime_lock: true,
            vars: vec![
                EnvVar {
                    key: "APP_ENV".to_string(),
                    default: "development".to_string(),
                    comment: "development | staging | production".to_string(),
                    secret: false,
                },
                EnvVar {
                    key: "APP_NAME".to_string(),
                    default: "{project}".to_string(),
                    comment: "application identifier".to_string(),
                    secret: false,
                },
                EnvVar {
                    key: "LOG_LEVEL".to_string(),
                    default: "debug".to_string(),
                    comment: "trace | debug | info | warn | error".to_string(),
                    secret: false,
                },
                EnvVar {
                    key: "PORT".to_string(),
                    default: "3000".to_string(),
                    comment: "primary server port".to_string(),
                    secret: false,
                },
            ],
            validation: EnvValidation::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub default: String,
    pub comment: String,
    pub secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvValidation {
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<EnvValidationRule>,
}

impl Default for EnvValidation {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: vec![
                EnvValidationRule {
                    key: "PORT".to_string(),
                    rule_type: "u16".to_string(),
                    values: None,
                },
                EnvValidationRule {
                    key: "APP_ENV".to_string(),
                    rule_type: "enum".to_string(),
                    values: Some(vec![
                        "development".to_string(),
                        "staging".to_string(),
                        "production".to_string(),
                    ]),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvValidationRule {
    pub key: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvFile {
    pub vars: BTreeMap<String, String>,
}

impl EnvFile {
    pub fn parse(content: &str) -> Self {
        let mut vars = BTreeMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                vars.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Self { vars }
    }

    pub fn render(&self, config: &EnvConfig) -> String {
        let mut lines = Vec::new();
        for var in &config.vars {
            if !var.comment.is_empty() {
                lines.push(format!("# {}", var.comment));
            }
            let value = if var.secret {
                "CHANGEME".to_string()
            } else {
                self.vars
                    .get(&var.key)
                    .cloned()
                    .unwrap_or_else(|| var.default.clone())
            };
            lines.push(format!("{}={}", var.key, value));
            lines.push(String::new());
        }
        lines.join("\n")
    }

    pub fn render_example(&self, config: &EnvConfig) -> String {
        let mut lines = Vec::new();
        for var in &config.vars {
            if !var.comment.is_empty() {
                lines.push(format!("# {}", var.comment));
            }
            let value = if var.secret {
                "CHANGEME".to_string()
            } else {
                self.vars
                    .get(&var.key)
                    .cloned()
                    .unwrap_or_else(|| var.default.clone())
            };
            lines.push(format!("{}={}", var.key, value));
            lines.push(String::new());
        }
        lines.join("\n")
    }
}

pub fn generate_env(project_dir: &Path, config: &EnvConfig, project_name: &str) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    let mut env_file = EnvFile::default();
    for var in &config.vars {
        let value = var.default.replace("{project}", project_name);
        env_file.vars.insert(var.key.clone(), value);
    }
    let content = env_file.render(config);
    root.write_atomic(".env", content)?;
    let example_content = env_file.render_example(config);
    root.write_atomic(".env.example", example_content)?;
    Ok(())
}

pub fn check_env_drift(project_dir: &Path, config: &EnvConfig) -> Result<Vec<EnvDrift>> {
    let env_path = project_dir.join(".env");
    if !env_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&env_path).map_err(|source| LodeError::Io {
        path: env_path.into(),
        source,
    })?;
    let actual = EnvFile::parse(&content);
    let mut drifts = Vec::new();
    for var in &config.vars {
        if !actual.vars.contains_key(&var.key) {
            drifts.push(EnvDrift {
                key: var.key.clone(),
                issue: "missing".to_string(),
            });
        }
    }
    Ok(drifts)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvDrift {
    pub key: String,
    pub issue: String,
}

pub fn validate_env(project_dir: &Path, config: &EnvConfig) -> Result<Vec<EnvValidationError>> {
    let env_path = project_dir.join(".env");
    if !env_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&env_path).map_err(|source| LodeError::Io {
        path: env_path.into(),
        source,
    })?;
    let actual = EnvFile::parse(&content);
    let mut errors = Vec::new();
    for rule in &config.validation.rules {
        if let Some(value) = actual.vars.get(&rule.key) {
            if !validate_value(value, &rule.rule_type, rule.values.as_deref()) {
                errors.push(EnvValidationError {
                    key: rule.key.clone(),
                    value: value.clone(),
                    rule: rule.rule_type.clone(),
                });
            }
        }
    }
    Ok(errors)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvValidationError {
    pub key: String,
    pub value: String,
    pub rule: String,
}

fn validate_value(value: &str, rule_type: &str, allowed: Option<&[String]>) -> bool {
    match rule_type {
        "u16" => value.parse::<u16>().is_ok(),
        "enum" => {
            if let Some(values) = allowed {
                values.iter().any(|v| v == value)
            } else {
                true
            }
        }
        _ => true,
    }
}

pub fn generate_runtime_lock(project_dir: &Path, language: &str, version: &str) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    match language {
        "rust" => {
            let content = format!("[toolchain]\nchannel = \"{version}\"\n");
            root.write_atomic("rust-toolchain.toml", content)?;
        }
        "node" | "typescript" => {
            root.write_atomic(".nvmrc", format!("{version}\n"))?;
        }
        "python" => {
            root.write_atomic(".python-version", format!("{version}\n"))?;
        }
        "go" => {
            root.write_atomic(".go-version", format!("{version}\n"))?;
        }
        "ruby" => {
            root.write_atomic(".ruby-version", format!("{version}\n"))?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_file_parses() {
        let content = "APP_ENV=development\nPORT=3000\n";
        let env = EnvFile::parse(content);
        assert_eq!(env.vars.get("APP_ENV").unwrap(), "development");
        assert_eq!(env.vars.get("PORT").unwrap(), "3000");
    }

    #[test]
    fn validate_u16() {
        assert!(validate_value("3000", "u16", None));
        assert!(!validate_value("abc", "u16", None));
        assert!(!validate_value("99999", "u16", None));
    }

    #[test]
    fn validate_enum() {
        let values = vec!["dev".to_string(), "prod".to_string()];
        assert!(validate_value("dev", "enum", Some(&values)));
        assert!(!validate_value("staging", "enum", Some(&values)));
    }
}
