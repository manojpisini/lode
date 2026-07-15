#![deny(unsafe_code)]

use crate::cmd::output;
use crate::OutputFormat;
use lode_core::{command_names, global_dir, profile_names, template_paths};

pub fn info_with_output(output: OutputFormat) -> lode_core::Result<()> {
    let dir = global_dir()?;
    if output.should_use_json() {
        println!(
            "{{\"config\":\"{}\",\"profiles\":{},\"templates\":{},\"commands\":{}}}",
            dir.join("config.toml"),
            profile_names().len(),
            template_paths().len(),
            command_names().len()
        );
    } else {
        println!("{}", output::section("Lode Info"));
        let rows = vec![
            vec!["config".to_string(), dir.join("config.toml").to_string()],
            vec!["profiles".to_string(), profile_names().len().to_string()],
            vec!["templates".to_string(), template_paths().len().to_string()],
            vec!["commands".to_string(), command_names().len().to_string()],
        ];
        print!("{}", output::table(&["resource", "value"], &rows));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_fn_exists() {
        let _fn: fn(OutputFormat) -> lode_core::Result<()> = info_with_output;
    }
}
