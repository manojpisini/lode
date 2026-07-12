use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionRule {
    pub name: String,
    pub pattern: String,
    pub case: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesConfig {
    pub rules: Vec<ConventionRule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleViolation {
    pub rule_name: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RulesReport {
    pub checked: usize,
    pub violations: Vec<RuleViolation>,
}

pub fn load_rules(project_dir: &Path) -> Result<RulesConfig> {
    let rules_path = project_dir.join(".lode").join("rules.toml");
    if !rules_path.exists() {
        return Ok(RulesConfig::default());
    }

    let content = std::fs::read_to_string(&rules_path).map_err(|source| LodeError::Io {
        path: rules_path.clone(),
        source,
    })?;

    let config: RulesConfig =
        toml::from_str(&content).map_err(|source| LodeError::TomlDeserialize {
            path: rules_path,
            source: Box::new(source),
        })?;

    Ok(config)
}

fn is_case_valid(name: &str, case: &str) -> bool {
    match case {
        "kebab-case" => {
            let re =
                Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("hardcoded pattern: kebab-case");
            re.is_match(name)
        }
        "snake_case" => {
            let re =
                Regex::new(r"^[a-z0-9]+(_[a-z0-9]+)*$").expect("hardcoded pattern: snake_case");
            re.is_match(name)
        }
        "PascalCase" => {
            let re = Regex::new(r"^[A-Z][a-zA-Z0-9]*$").expect("hardcoded pattern: PascalCase");
            re.is_match(name)
        }
        "camelCase" => {
            let re = Regex::new(r"^[a-z][a-zA-Z0-9]*$").expect("hardcoded pattern: camelCase");
            re.is_match(name)
        }
        "SCREAMING_SNAKE_CASE" => {
            let re = Regex::new(r"^[A-Z0-9]+(_[A-Z0-9]+)*$")
                .expect("hardcoded pattern: SCREAMING_SNAKE_CASE");
            re.is_match(name)
        }
        _ => true,
    }
}

pub fn check_rules(project_dir: &Path, config: &RulesConfig) -> Result<RulesReport> {
    let mut report = RulesReport {
        checked: 0,
        violations: Vec::new(),
    };

    if !project_dir.exists() {
        return Ok(report);
    }

    visit_dir(project_dir, project_dir, config, &mut report)?;
    Ok(report)
}

fn visit_dir(
    base: &Path,
    current: &Path,
    config: &RulesConfig,
    report: &mut RulesReport,
) -> Result<()> {
    if !current.exists() {
        return Ok(());
    }

    if let Some(name) = current.file_name().and_then(|n| n.to_str()) {
        if should_prune(name) {
            return Ok(());
        }

        for rule in &config.rules {
            let re = Regex::new(&rule.pattern).map_err(|e| {
                LodeError::Message(format!("invalid regex in rule '{}': {}", rule.name, e))
            })?;

            let check_name = Path::new(name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(name);
            if re.is_match(name) && !is_case_valid(check_name, &rule.case) {
                let relative = current
                    .strip_prefix(base)
                    .unwrap_or(current)
                    .to_string_lossy()
                    .to_string();
                report.violations.push(RuleViolation {
                    rule_name: rule.name.clone(),
                    path: relative,
                    message: format!(
                        "file '{}' matched rule '{}' but is not in {}",
                        name, rule.name, rule.case
                    ),
                });
            }
        }

        report.checked += 1;
    }

    if current.is_dir() {
        let entries = std::fs::read_dir(current).map_err(|source| LodeError::Io {
            path: current.to_path_buf(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| LodeError::Io {
                path: current.to_path_buf(),
                source,
            })?;
            visit_dir(base, &entry.path(), config, report)?;
        }
    }

    Ok(())
}

pub fn validate_rules(rules: &[ConventionRule]) -> Result<()> {
    for rule in rules {
        Regex::new(&rule.pattern).map_err(|e| {
            LodeError::Message(format!("invalid regex in rule '{}': {}", rule.name, e))
        })?;

        match rule.case.as_str() {
            "kebab-case" | "snake_case" | "PascalCase" | "camelCase" | "SCREAMING_SNAKE_CASE" => {}
            other => {
                return Err(LodeError::Message(format!(
                    "unsupported case '{}' in rule '{}': expected kebab-case, snake_case, PascalCase, camelCase, or SCREAMING_SNAKE_CASE",
                    other, rule.name,
                )));
            }
        }
    }
    Ok(())
}

fn should_prune(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | "__pycache__" | ".venv" | "dist" | "build"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_config() -> RulesConfig {
        RulesConfig {
            rules: vec![
                ConventionRule {
                    name: "rust_files".to_string(),
                    pattern: r"\.rs$".to_string(),
                    case: "snake_case".to_string(),
                    description: "Rust source files must be snake_case".to_string(),
                },
                ConventionRule {
                    name: "ts_files".to_string(),
                    pattern: r"\.ts$".to_string(),
                    case: "camelCase".to_string(),
                    description: "TypeScript files must be camelCase".to_string(),
                },
            ],
        }
    }

    #[test]
    fn validate_rules_rejects_bad_regex() {
        let rules = vec![ConventionRule {
            name: "bad".to_string(),
            pattern: "[invalid".to_string(),
            case: "snake_case".to_string(),
            description: "bad".to_string(),
        }];
        assert!(validate_rules(&rules).is_err());
    }

    #[test]
    fn validate_rules_rejects_unknown_case() {
        let rules = vec![ConventionRule {
            name: "bad".to_string(),
            pattern: r"\.rs$".to_string(),
            case: "UNKNOWN_CASE".to_string(),
            description: "bad".to_string(),
        }];
        assert!(validate_rules(&rules).is_err());
    }

    #[test]
    fn validate_rules_accepts_valid_rules() {
        let rules = vec![ConventionRule {
            name: "good".to_string(),
            pattern: r"\.rs$".to_string(),
            case: "snake_case".to_string(),
            description: "Rust files".to_string(),
        }];
        assert!(validate_rules(&rules).is_ok());
    }

    #[test]
    fn check_rules_catches_violations() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("BadFileName.rs"), "").unwrap();
        let config = sample_config();
        let report = check_rules(dir.path(), &config).unwrap();
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].rule_name, "rust_files");
    }

    #[test]
    fn check_rules_allows_valid_names() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("good_file.rs"), "").unwrap();
        let config = sample_config();
        let report = check_rules(dir.path(), &config).unwrap();
        assert!(report.violations.is_empty());
    }

    #[test]
    fn load_rules_returns_empty_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_rules(dir.path()).unwrap();
        assert!(config.rules.is_empty());
    }

    #[test]
    fn load_rules_reads_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let lode_dir = dir.path().join(".lode");
        fs::create_dir(&lode_dir).unwrap();
        fs::write(
            lode_dir.join("rules.toml"),
            r#"[[rules]]
name = "test_rule"
pattern = '\.rs$'
case = "snake_case"
description = "test"
"#,
        )
        .unwrap();
        let config = load_rules(dir.path()).unwrap();
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "test_rule");
    }
}
