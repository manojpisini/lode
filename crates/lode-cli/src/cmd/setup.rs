#![deny(unsafe_code)]

use lode_core::setup_defaults;

pub fn setup() -> lode_core::Result<()> {
    let report = setup_defaults(false)?;
    println!("lode initialised at {}", report.global_dir);
    println!(
        "{} {}",
        if report.wrote_config {
            "wrote"
        } else {
            "kept existing"
        },
        report.config_path
    );
    println!(
        "extracted default templates, profiles, snippets, recipes, licenses, and command macros"
    );
    Ok(())
}
