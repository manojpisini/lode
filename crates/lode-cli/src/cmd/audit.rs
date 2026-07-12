#![deny(unsafe_code)]

use lode_core::{load_global_config, audit_project, save_metrics};

pub fn health() -> lode_core::Result<()> {
    let cwd = crate::current_dir()?;
    let config = load_global_config()?;
    let report = audit_project(&cwd, &config)?;
    let metrics_path = save_metrics(&cwd, &report)?;
    println!("health score: {}", report.score);
    println!("convention violations: {}", report.convention_violations);
    println!("secret findings: {}", report.secret_findings);
    println!("license: {}", crate::status_bool(report.license_present));
    println!("env example: {}", crate::status_bool(report.env_example_present));
    println!("readme: {}", crate::status_bool(report.readme_present));
    println!("metrics: {metrics_path}");
    Ok(())
}
