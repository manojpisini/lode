#![deny(unsafe_code)]

use crate::MetricsCommand;
use lode_core::{audit_project, load_global_config, load_metrics, save_metrics};

pub(crate) fn metrics(command: MetricsCommand) -> lode_core::Result<()> {
    match command {
        MetricsCommand::Show => {
            let report = load_metrics(&crate::current_dir()?)?;
            println!("metrics score: {}", report.score);
            println!("convention violations: {}", report.convention_violations);
            println!("secret findings: {}", report.secret_findings);
        }
        MetricsCommand::Trend { last } => {
            let report = load_metrics(&crate::current_dir()?)?;
            println!("metrics trend: latest score {}", report.score);
            if let Some(last) = last {
                println!("window: last {last} snapshot(s)");
            }
        }
        MetricsCommand::Baseline => {
            let cwd = crate::current_dir()?;
            let report = audit_project(&cwd, &load_global_config()?)?;
            save_metrics(&cwd, &report)?;
            crate::save_metrics_baseline(&cwd, &report)?;
            println!("metrics baseline saved");
        }
        MetricsCommand::DiffBaseline => {
            let cwd = crate::current_dir()?;
            let current = load_metrics(&cwd)?;
            let baseline = crate::load_metrics_baseline(&cwd)?;
            println!(
                "score delta: {}",
                current.score as i16 - baseline.score as i16
            );
            println!(
                "convention delta: {}",
                current.convention_violations as i64 - baseline.convention_violations as i64
            );
            println!(
                "secret delta: {}",
                current.secret_findings as i64 - baseline.secret_findings as i64
            );
        }
    }
    Ok(())
}
