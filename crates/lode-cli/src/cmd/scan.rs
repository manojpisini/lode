#![deny(unsafe_code)]

use crate::cmd::output as out;
use crate::ScanCommand;
use lode_core::scan_secrets;

pub(crate) fn scan(command: ScanCommand) -> lode_core::Result<()> {
    match command {
        ScanCommand::Secrets {
            path,
            staged,
            output,
            quiet,
        } => {
            let path = path.unwrap_or(crate::current_dir()?);
            if staged {
                println!("  {} {}", out::info("staged"), out::dim(path.as_str()));
            }
            let report = scan_secrets(&path)?;
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| lode_core::LodeError::Message(error.to_string()))?
                );
            } else if !quiet {
                println!("{}", out::section("Secret Scan"));
                if report.findings.is_empty() {
                    println!("  {} {}", out::ok("clean"), out::dim(path.as_str()));
                } else {
                    for finding in &report.findings {
                        println!(
                            "  {} {}:{} [{}]",
                            out::fail(""),
                            finding.path,
                            finding.line,
                            out::yellow(&finding.kind)
                        );
                    }
                }
            }
            if !report.findings.is_empty() {
                return Err(lode_core::LodeError::SecretFindings {
                    count: report.findings.len(),
                });
            }
        }
        ScanCommand::Foreign { path, output } => {
            let path = path.unwrap_or(crate::current_dir()?);
            let report = crate::scan_foreign_project(&path)?;
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| lode_core::LodeError::Message(error.to_string()))?
                );
            } else {
                println!("{}", out::section("Foreign Project Scan"));
                let lode_badge = if report.lode_project {
                    out::green("yes")
                } else {
                    out::red("no")
                };
                let rows = vec![
                    vec!["path".to_string(), report.path.to_string()],
                    vec!["lode project".to_string(), lode_badge],
                    vec![
                        "package manager".to_string(),
                        report
                            .package_manager
                            .as_deref()
                            .unwrap_or("none")
                            .to_string(),
                    ],
                    vec!["manifests".to_string(), report.manifests.join(", ")],
                    vec![
                        "convention violations".to_string(),
                        report.convention_violations.to_string(),
                    ],
                    vec![
                        "secret findings".to_string(),
                        report.secret_findings.to_string(),
                    ],
                ];
                print!("{}", out::table(&["field", "value"], &rows));
                for action in &report.migration_actions {
                    println!("  {} {}", out::info("action"), action);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OutputFormat, ScanCommand};

    #[test]
    fn test_scan_command_secrets_default() {
        let command = ScanCommand::Secrets {
            path: None,
            staged: false,
            output: OutputFormat::Table,
            quiet: false,
        };
        match command {
            ScanCommand::Secrets {
                path,
                staged,
                output,
                quiet,
            } => {
                assert!(path.is_none());
                assert!(!staged);
                assert!(!output.should_use_json());
                assert!(!quiet);
            }
            _ => panic!("expected Secrets variant"),
        }
    }

    #[test]
    fn test_scan_command_foreign() {
        let command = ScanCommand::Foreign {
            path: None,
            output: OutputFormat::Table,
        };
        match command {
            ScanCommand::Foreign { path, output } => {
                assert!(path.is_none());
                assert!(!output.should_use_json());
            }
            _ => panic!("expected Foreign variant"),
        }
    }

    #[test]
    fn test_scan_fn_exists() {
        let _fn: fn(ScanCommand) -> lode_core::Result<()> = scan;
    }
}
