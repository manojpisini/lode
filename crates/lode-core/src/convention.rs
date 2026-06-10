use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;

use crate::{config::LodeConfig, LodeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConventionViolation {
    pub path: Utf8PathBuf,
    pub expected_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConventionReport {
    pub checked: usize,
    pub violations: Vec<ConventionViolation>,
    pub renamed: Vec<(Utf8PathBuf, Utf8PathBuf)>,
}

impl ConventionReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

pub fn check_path(path: &Utf8Path, config: &LodeConfig) -> Result<ConventionReport> {
    let mut report = ConventionReport {
        checked: 0,
        violations: Vec::new(),
        renamed: Vec::new(),
    };
    visit(path, config, &mut report)?;
    Ok(report)
}

pub fn fix_path(path: &Utf8Path, config: &LodeConfig) -> Result<ConventionReport> {
    let mut report = check_path(path, config)?;
    let mut targets = report.violations.clone();
    targets.sort_by_key(|violation| std::cmp::Reverse(violation.path.components().count()));

    for violation in targets {
        let Some(parent) = violation.path.parent() else {
            continue;
        };
        let destination = parent.join(&violation.expected_name);
        if destination == violation.path || destination.exists() {
            continue;
        }
        fs::rename(&violation.path, &destination).map_err(|source| LodeError::Io {
            path: PathBuf::from(violation.path.as_str()),
            source,
        })?;
        report.renamed.push((violation.path, destination));
    }

    report.violations.clear();
    Ok(report)
}

pub fn normalize_name(name: &str, config: &LodeConfig) -> String {
    if should_skip_name(name, config) {
        return name.to_string();
    }

    let (stem, extension) = split_name(name);
    let words = split_words(stem);
    let normalized_stem = match config.convention.default_case.as_str() {
        "kebab-case" => words.join("-"),
        "PascalCase" => words
            .iter()
            .map(|word| capitalize(word))
            .collect::<Vec<_>>()
            .join(""),
        "camelCase" => {
            let mut output = String::new();
            for (index, word) in words.iter().enumerate() {
                if index == 0 {
                    output.push_str(word);
                } else {
                    output.push_str(&capitalize(word));
                }
            }
            output
        }
        _ => words.join("_"),
    };

    if let Some(extension) = extension {
        format!("{normalized_stem}.{extension}")
    } else {
        normalized_stem
    }
}

fn visit(path: &Utf8Path, config: &LodeConfig, report: &mut ConventionReport) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if let Some(name) = path.file_name() {
        if should_prune(name) {
            return Ok(());
        }
        report.checked += 1;
        let expected = normalize_name(name, config);
        if expected != name {
            report.violations.push(ConventionViolation {
                path: path.to_path_buf(),
                expected_name: expected,
            });
        }
    }

    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: PathBuf::from(path.as_str()),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: PathBuf::from(path.as_str()),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            visit(&child, config, report)?;
        }
    }

    Ok(())
}

fn split_name(name: &str) -> (&str, Option<&str>) {
    if name.starts_with('.') {
        return (name, None);
    }
    name.rsplit_once('.')
        .map_or((name, None), |(stem, extension)| (stem, Some(extension)))
}

fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && previous_lower && !current.is_empty() {
                words.push(current);
                current = String::new();
            }
            current.push(ch.to_ascii_lowercase());
            previous_lower = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else if !current.is_empty() {
            words.push(current);
            current = String::new();
            previous_lower = false;
        }
    }

    if !current.is_empty() {
        words.push(current);
    }
    if words.is_empty() {
        vec![input.to_ascii_lowercase()]
    } else {
        words
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn should_skip_name(name: &str, config: &LodeConfig) -> bool {
    if name == "." || name == ".." || name.starts_with('.') {
        return true;
    }
    if config
        .convention
        .protected_prefixes
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return true;
    }
    matches!(
        name,
        "README.md"
            | "CHANGELOG.md"
            | "CONTRIBUTING.md"
            | "LICENSE"
            | "Makefile"
            | "Dockerfile"
            | "AGENTS.md"
            | "CLAUDE.md"
            | "CODEX.md"
    )
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
    use crate::config;

    #[test]
    fn normalizes_names_to_snake_case() {
        assert_eq!(
            normalize_name("MyHTTPServer.rs", &config::default_config()),
            "my_httpserver.rs"
        );
        assert_eq!(
            normalize_name("some-file name.ts", &config::default_config()),
            "some_file_name.ts"
        );
    }

    #[test]
    fn reports_violations() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        fs::write(root.join("BadName.rs"), "").unwrap();

        let report = check_path(&root, &config::default_config()).unwrap();

        assert_eq!(report.violations.len(), 1);
    }
}
