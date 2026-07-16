#![deny(unsafe_code)]

use crate::cmd::output;
use crate::OutputFormat;
use lode_core::setup_defaults;
use serde_json::json;

pub(crate) fn doctor_with_output(fix: bool, output: OutputFormat) -> lode_core::Result<()> {
    let mut fixed = false;
    if fix {
        setup_defaults(false)?;
        fixed = true;
    }
    let report = crate::build_doctor_report(fixed);
    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!(report))
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        let badge = match report.status.as_str() {
            "ok" => output::ok("doctor"),
            "warn" => output::warn("doctor"),
            _ => output::fail("doctor"),
        };
        println!("{}", output::section("System Check"));
        println!("  {}\n", badge);
        if report.fixed {
            println!(
                "  {} {}\n",
                output::green("✔"),
                output::dim("safe defaults refreshed")
            );
        }
        let rows: Vec<Vec<String>> = report
            .checks
            .iter()
            .map(|c| {
                let sym = match c.status.as_str() {
                    "ok" => output::green("✔"),
                    "warn" => output::yellow("⚠"),
                    _ => output::red("✘"),
                };
                vec![sym, c.name.clone(), c.detail.clone()]
            })
            .collect();
        print!("{}", output::table(&["", "check", "detail"], &rows));
    }
    Ok(())
}
