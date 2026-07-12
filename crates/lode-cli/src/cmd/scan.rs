#![deny(unsafe_code)]

use crate::ScanCommand;
use lode_core::scan_secrets;

pub(crate) fn scan(command: ScanCommand) -> lode_core::Result<()> {
    match command {
        ScanCommand::Secrets {
            path,
            staged,
            json,
            quiet,
        } => {
            let path = path.unwrap_or(crate::current_dir()?);
            if staged {
                println!("scanning staged-compatible project path: {path}");
            }
            let report = scan_secrets(&path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| lode_core::LodeError::Message(error.to_string()))?
                );
            } else if !quiet {
                if report.findings.is_empty() {
                    println!("no obvious secrets found in {path}");
                } else {
                    for finding in &report.findings {
                        println!("{}:{} {}", finding.path, finding.line, finding.kind);
                    }
                }
            }
            if !report.findings.is_empty() {
                return Err(lode_core::LodeError::SecretFindings {
                    count: report.findings.len(),
                });
            }
        }
        ScanCommand::Foreign { path, json } => {
            let path = path.unwrap_or(crate::current_dir()?);
            let report = crate::scan_foreign_project(&path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| lode_core::LodeError::Message(error.to_string()))?
                );
            } else {
                println!("foreign project scan: {}", report.path);
                println!("lode_project\t{}", crate::status_bool(report.lode_project));
                println!(
                    "package_manager\t{}",
                    report.package_manager.as_deref().unwrap_or("none")
                );
                println!("manifests\t{}", report.manifests.join(","));
                println!("convention_violations\t{}", report.convention_violations);
                println!("secret_findings\t{}", report.secret_findings);
                for action in &report.migration_actions {
                    println!("action\t{action}");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScanCommand;

    #[test]
    fn test_scan_command_secrets_default() {
        let command = ScanCommand::Secrets {
            path: None,
            staged: false,
            json: false,
            quiet: false,
        };
        match command {
            ScanCommand::Secrets { path, staged, json, quiet } => {
                assert!(path.is_none());
                assert!(!staged);
                assert!(!json);
                assert!(!quiet);
            }
            _ => panic!("expected Secrets variant"),
        }
    }

    #[test]
    fn test_scan_command_foreign() {
        let command = ScanCommand::Foreign {
            path: None,
            json: false,
        };
        match command {
            ScanCommand::Foreign { path, json } => {
                assert!(path.is_none());
                assert!(!json);
            }
            _ => panic!("expected Foreign variant"),
        }
    }

    #[test]
    fn test_scan_fn_exists() {
        let _fn: fn(ScanCommand) -> lode_core::Result<()> = scan;
    }
}
