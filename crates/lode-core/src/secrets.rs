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
    matches!(
        name,
        ".env" | ".env.local" | ".env.development" | ".env.production" | ".env.test"
    )
}

fn classify_secret(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    let has_assignment = line.contains('=') || line.contains(':');
    if has_assignment
        && [
            "api_key",
            "apikey",
            "secret",
            "token",
            "password",
            "private_key",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        && !lower.contains("changeme")
        && !lower.contains("example")
    {
        return Some("suspicious credential assignment");
    }
    if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----") {
        return Some("private key");
    }
    if line.contains("ghp_") || line.contains("github_pat_") {
        return Some("github token");
    }
    if (line.contains("AKIA")
        || line.contains("ASIA")
        || line.contains("ABIA")
        || line.contains("ACCA")
        || line.contains("AROA"))
        && line.len() >= 20
    {
        return Some("aws access key");
    }
    None
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
    fn skips_real_env_files_but_scans_examples() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        fs::write(root.join(".env"), "API_KEY=real-value\n").unwrap();
        fs::write(root.join(".env.example"), "API_KEY=real-value\n").unwrap();

        let report = scan_secrets(&root).unwrap();

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].path.file_name(), Some(".env.example"));
    }
}
