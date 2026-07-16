#![deny(unsafe_code)]

use crate::cmd::output;
use crate::OutputFormat;
use lode_core::setup_defaults;
use serde_json::json;

pub(crate) fn setup_with_output(output: OutputFormat) -> lode_core::Result<()> {
    let report = setup_defaults(false)?;
    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "global_dir": report.global_dir,
                "config_path": report.config_path,
                "created_dirs": report.created_dirs,
                "wrote_config": report.wrote_config,
            }))
            .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!("{}", output::section("Setup"));
        println!(
            "  {} {}",
            output::ok("init"),
            output::dim(report.global_dir.as_str())
        );
        let config_status = if report.wrote_config {
            output::ok("wrote")
        } else {
            output::info("kept existing")
        };
        println!(
            "  {} {}",
            config_status,
            output::dim(report.config_path.as_str())
        );
        println!(
            "  {} {}",
            output::info("extracted"),
            output::dim("templates, profiles, snippets, recipes, licenses, command macros")
        );
    }
    Ok(())
}
