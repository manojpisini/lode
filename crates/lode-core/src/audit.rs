use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{check_path, scan_secrets, LodeConfig, LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReport {
    pub score: u8,
    pub convention_violations: usize,
    pub secret_findings: usize,
    pub license_present: bool,
    pub env_example_present: bool,
    pub readme_present: bool,
}

pub fn audit_project(path: &Utf8Path, config: &LodeConfig) -> Result<AuditReport> {
    let convention = check_path(path, config)?;
    let secrets = scan_secrets(path)?;
    let license_present = path.join("LICENSE").exists();
    let env_example_present = path.join(".env.example").exists();
    let readme_present = path.join("README.md").exists();
    let mut score = 100i32;
    score -= (convention.violations.len() as i32).min(20);
    score -= (secrets.findings.len() as i32 * 10).min(30);
    if !license_present {
        score -= 10;
    }
    if !env_example_present {
        score -= 5;
    }
    if !readme_present {
        score -= 5;
    }
    Ok(AuditReport {
        score: score.clamp(0, 100) as u8,
        convention_violations: convention.violations.len(),
        secret_findings: secrets.findings.len(),
        license_present,
        env_example_present,
        readme_present,
    })
}

pub fn save_metrics(path: &Utf8Path, report: &AuditReport) -> Result<Utf8PathBuf> {
    let root = ValidatedRoot::new(path)?;
    root.create_dir_all(".lode")?;
    let raw = serde_json::to_string_pretty(report)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic(".lode/metrics.json", raw)?;
    Ok(path.join(".lode").join("metrics.json"))
}

pub fn load_metrics(path: &Utf8Path) -> Result<AuditReport> {
    let metrics_path = path.join(".lode").join("metrics.json");
    let raw = fs::read_to_string(&metrics_path).map_err(|source| LodeError::Io {
        path: PathBuf::from(metrics_path.as_str()),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_config;

    #[test]
    fn audit_scores_basic_project() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        fs::write(root.join("README.md"), "").unwrap();
        fs::write(root.join("LICENSE"), "MIT").unwrap();
        fs::write(root.join(".env.example"), "APP_NAME=test").unwrap();

        let report = audit_project(&root, &default_config()).unwrap();

        assert!(report.score >= 90);
    }
}
