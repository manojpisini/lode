use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;

use crate::{LodeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecretFinding {
    pub path: Utf8PathBuf,
    pub line: usize,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecretScanReport {
    pub checked_files: usize,
    pub findings: Vec<SecretFinding>,
}

/// Scan a single text buffer in-memory (no file I/O) and return findings.
pub fn scan_content(content: &str) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if let Some(kind) = classify_secret(line) {
            findings.push(SecretFinding {
                path: Utf8PathBuf::from("<buffer>"),
                line: index + 1,
                kind: kind.to_string(),
            });
        }
    }
    findings
}

pub fn scan_secrets(path: &Utf8Path) -> Result<SecretScanReport> {
    let mut report = SecretScanReport {
        checked_files: 0,
        findings: Vec::new(),
    };
    visit(path, &mut report)?;
    Ok(report)
}

fn visit(path: &Utf8Path, report: &mut SecretScanReport) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        if path
            .file_name()
            .is_some_and(|name| matches!(name, ".git" | "target" | "node_modules" | ".venv"))
        {
            return Ok(());
        }
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
            visit(&child, report)?;
        }
        return Ok(());
    }

    if !is_text_candidate(path) {
        return Ok(());
    }
    if is_secret_allowlisted_path(path) {
        return Ok(());
    }
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("lode: warning: skipping unreadable file {}: {}", path, e);
            return Ok(());
        }
    };
    report.checked_files += 1;
    for (index, line) in contents.lines().enumerate() {
        if let Some(kind) = classify_secret(line) {
            report.findings.push(SecretFinding {
                path: path.to_path_buf(),
                line: index + 1,
                kind: kind.to_string(),
            });
        }
    }
    Ok(())
}

fn is_secret_allowlisted_path(path: &Utf8Path) -> bool {
    let Some(name) = path.file_name() else {
        return false;
    };
    matches!(name, ".env.example" | ".env.dist")
}

fn classify_secret(line: &str) -> Option<&'static str> {
    if line.trim_start().starts_with("-----BEGIN") && line.contains("PRIVATE KEY-----") {
        return Some("private key");
    }
    if contains_github_token(line) {
        return Some("github token");
    }
    if contains_aws_access_key(line) {
        return Some("aws access key");
    }

    let (raw_key, raw_value) = split_assignment(line)?;
    let key = raw_key.trim().trim_matches('"');
    let value = raw_value.trim().trim_matches('"').trim_matches(',');
    let lower_key = key.to_ascii_lowercase();
    let lower_value = value.to_ascii_lowercase();

    if key.is_empty() || key.len() > 80 || key.chars().any(char::is_whitespace) {
        return None;
    }

    if is_secret_key(&lower_key) && looks_like_secret_value(value, &lower_value) {
        return Some("suspicious credential assignment");
    }
    None
}

fn contains_github_token(line: &str) -> bool {
    line.split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '`' | ',' | ';'))
        .any(|part| {
            let trimmed = part.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
            (trimmed.starts_with("ghp_") && trimmed.len() >= 40)
                || (trimmed.starts_with("github_pat_") && trimmed.len() >= 40)
        })
}

fn contains_aws_access_key(line: &str) -> bool {
    if line.contains("redact(") {
        return false;
    }
    line.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|part| {
            part.len() == 20
                && ["AKIA", "ASIA", "ABIA", "ACCA", "AROA"]
                    .iter()
                    .any(|prefix| part.starts_with(prefix))
                && part
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        })
}

fn looks_like_secret_value(value: &str, lower_value: &str) -> bool {
    if is_placeholder_secret_value(lower_value) {
        return false;
    }
    let value = value.trim();
    let distinct = value
        .chars()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    distinct >= 6
        && value.len() >= 8
        && !value.chars().any(char::is_whitespace)
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '+' | '='))
}
fn is_secret_key(lower_key: &str) -> bool {
    if lower_key.contains("finding") || lower_key.contains("found") || lower_key.contains("count") {
        return false;
    }
    lower_key == "api_key"
        || lower_key == "apikey"
        || lower_key == "password"
        || lower_key == "private_key"
        || lower_key.ends_with("_api_key")
        || lower_key.ends_with("_token")
        || lower_key.ends_with("_secret")
        || lower_key.ends_with("_password")
}

fn split_assignment(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("<!--")
    {
        return None;
    }
    if let Some(pair) = trimmed.split_once('=') {
        return Some(pair);
    }
    trimmed.split_once(':')
}

fn is_placeholder_secret_value(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.len() < 8 {
        return true;
    }
    for needle in [
        "changeme",
        "change_me",
        "example",
        "placeholder",
        "dummy",
        "fake",
        "test",
        "todo",
        "redacted",
        "<",
        "your_",
        "none",
        "null",
    ] {
        if value.contains(needle) {
            return true;
        }
    }
    false
}
fn is_text_candidate(path: &Utf8Path) -> bool {
    if path
        .file_name()
        .is_some_and(|name| name.starts_with(".env"))
    {
        return true;
    }
    path.extension().is_none_or(|extension| {
        matches!(
            extension,
            "env"
                | "txt"
                | "md"
                | "toml"
                | "json"
                | "yaml"
                | "yml"
                | "rs"
                | "ts"
                | "js"
                | "py"
                | "go"
                | "java"
                | "c"
                | "cpp"
                | "h"
                | "hpp"
                | "sh"
                | "ps1"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_suspicious_assignments() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        fs::write(root.join("src.rs"), "API_KEY=real-value\n").unwrap();

        let report = scan_secrets(&root).unwrap();

        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn scans_env_files_and_skips_examples() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        fs::write(root.join(".env"), "API_KEY=real-value\n").unwrap();
        fs::write(root.join(".env.example"), "API_KEY=real-value\n").unwrap();

        let report = scan_secrets(&root).unwrap();

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].path.file_name(), Some(".env"));
    }
}
