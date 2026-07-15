use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub scope: Vec<String>,
    pub check: PolicyCheck,
    #[serde(default)]
    pub remediation: Option<PolicyRemediation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheck {
    pub kind: String,
    #[serde(default)]
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRemediation {
    pub recipe: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWaiver {
    pub policy_id: String,
    pub reason: String,
    pub expires: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyReport {
    pub checked: usize,
    pub passed: usize,
    pub failed: usize,
    pub waivers: Vec<PolicyWaiver>,
    pub results: Vec<PolicyCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckResult {
    pub policy_id: String,
    pub severity: String,
    pub passed: bool,
    pub waived: bool,
    pub message: String,
}

pub fn load_policies() -> Result<Vec<Policy>> {
    let dir = crate::install::global_asset_dir("policies")?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut policies = Vec::new();
    for entry in fs::read_dir(dir.as_std_path()).map_err(|source| LodeError::Io {
        path: dir.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.as_str().into(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.clone(),
            source,
        })?;
        match toml::from_str::<Policy>(&content) {
            Ok(p) => policies.push(p),
            Err(e) => {
                eprintln!(
                    "lode: warning: policy file {} parse error: {}",
                    path.display(),
                    e
                );
            }
        }
    }
    Ok(policies)
}

pub fn check_policies(policies: &[Policy], waivers: &[PolicyWaiver]) -> PolicyReport {
    let mut results = Vec::new();
    let mut failed = 0usize;
    let mut passed = 0usize;

    for policy in policies {
        let waived = waivers.iter().any(|w| w.policy_id == policy.id);
        let result = match policy.check.kind.as_str() {
            "secret-scan" => {
                let message = "check requires project context".to_string();
                let ok = false;
                PolicyCheckResult {
                    policy_id: policy.id.clone(),
                    severity: policy.severity.clone(),
                    passed: ok,
                    waived,
                    message,
                }
            }
            "always-fail" => PolicyCheckResult {
                policy_id: policy.id.clone(),
                severity: policy.severity.clone(),
                passed: false,
                waived,
                message: "policy check not yet implemented".to_string(),
            },
            kind => {
                let ok = false;
                PolicyCheckResult {
                    policy_id: policy.id.clone(),
                    severity: policy.severity.clone(),
                    passed: ok,
                    waived,
                    message: format!("unknown check kind: {kind}"),
                }
            }
        };
        if result.passed || result.waived {
            passed += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }

    PolicyReport {
        checked: results.len(),
        passed,
        failed,
        waivers: waivers.to_vec(),
        results,
    }
}

pub fn load_waivers(project_dir: &camino::Utf8Path) -> Result<Vec<PolicyWaiver>> {
    let path = project_dir.join(".lode").join("policy-waivers.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path.as_std_path()).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))
}

pub fn save_waivers(project_dir: &camino::Utf8Path, waivers: &[PolicyWaiver]) -> Result<()> {
    let path = project_dir.join(".lode").join("policy-waivers.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent.as_std_path()).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let json =
        serde_json::to_string_pretty(waivers).map_err(|e| LodeError::Message(e.to_string()))?;
    fs::write(path.as_std_path(), &json).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_policies() -> Vec<Policy> {
        vec![
            Policy {
                id: "security.no-plaintext-secrets".into(),
                severity: "error".into(),
                scope: vec![],
                check: PolicyCheck {
                    kind: "always-fail".into(),
                    config: HashMap::new(),
                },
                remediation: None,
            },
            Policy {
                id: "quality.format".into(),
                severity: "warning".into(),
                scope: vec![],
                check: PolicyCheck {
                    kind: "always-fail".into(),
                    config: HashMap::new(),
                },
                remediation: None,
            },
        ]
    }

    #[test]
    fn test_check_policies_all_fail() {
        let report = check_policies(&sample_policies(), &[]);
        assert_eq!(report.checked, 2);
        assert_eq!(report.failed, 2);
        assert_eq!(report.passed, 0);
    }

    #[test]
    fn test_check_policies_waived() {
        let waivers = vec![PolicyWaiver {
            policy_id: "security.no-plaintext-secrets".into(),
            reason: "accepted risk".into(),
            expires: None,
            owner: None,
        }];
        let report = check_policies(&sample_policies(), &waivers);
        assert_eq!(report.checked, 2);
        assert_eq!(report.failed, 1);
        assert_eq!(report.passed, 1);
    }

    #[test]
    fn test_check_unknown_kind() {
        let policies = vec![Policy {
            id: "custom".into(),
            severity: "info".into(),
            scope: vec![],
            check: PolicyCheck {
                kind: "unknown-check".into(),
                config: HashMap::new(),
            },
            remediation: None,
        }];
        let report = check_policies(&policies, &[]);
        assert_eq!(
            report.results[0].message,
            "unknown check kind: unknown-check"
        );
    }
}
