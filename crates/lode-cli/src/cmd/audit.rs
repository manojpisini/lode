#![deny(unsafe_code)]

use crate::cmd::output;
use crate::OutputFormat;
use lode_core::{audit_project, load_global_config, save_metrics};

pub fn health_with_output(output: OutputFormat) -> lode_core::Result<()> {
    let cwd = crate::current_dir()?;
    let config = load_global_config()?;
    let report = audit_project(&cwd, &config)?;
    let metrics_path = save_metrics(&cwd, &report)?;
    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!("{}", output::section("Health Report"));
        println!(
            "  {} {} {}",
            output::bold("Score:"),
            output::cyan(&report.score.to_string()),
            output::dim("/ 100")
        );
        println!(
            "  {}  {}",
            output::info("convention violations"),
            report.convention_violations
        );
        println!(
            "  {}  {}",
            output::info("secret findings"),
            report.secret_findings
        );
        println!(
            "  {}  {}",
            output::bold("license:"),
            if report.license_present {
                output::green("present")
            } else {
                output::red("missing")
            }
        );
        println!(
            "  {}  {}",
            output::bold("env example:"),
            if report.env_example_present {
                output::green("present")
            } else {
                output::red("missing")
            }
        );
        println!(
            "  {}  {}",
            output::bold("readme:"),
            if report.readme_present {
                output::green("present")
            } else {
                output::red("missing")
            }
        );
        println!(
            "  {}  {}",
            output::dim("metrics:"),
            output::dim(metrics_path.as_str())
        );
    }
    Ok(())
}
