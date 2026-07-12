#![deny(unsafe_code)]

use lode_core::setup_defaults;
use serde_json::json;

pub fn doctor(fix: bool, json: bool) -> lode_core::Result<()> {
    let mut fixed = false;
    if fix {
        setup_defaults(false)?;
        fixed = true;
    }
    let report = crate::build_doctor_report(fixed);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!(report))
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!("doctor {}", report.status);
        if report.fixed {
            println!("fixed\tsafe defaults refreshed");
        }
        for check in &report.checks {
            println!("{}\t{}\t{}", check.name, check.status, check.detail);
        }
    }
    Ok(())
}
