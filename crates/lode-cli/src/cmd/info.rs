#![deny(unsafe_code)]

use lode_core::{global_dir, profile_names, template_paths, command_names};

pub fn info(json: bool) -> lode_core::Result<()> {
    let dir = global_dir()?;
    if json {
        println!(
            "{{\"config\":\"{}\",\"profiles\":{},\"templates\":{},\"commands\":{}}}",
            dir.join("config.toml"),
            profile_names().len(),
            template_paths().len(),
            command_names().len()
        );
    } else {
        println!("config   {}", dir.join("config.toml"));
        println!("profiles {}", profile_names().len());
        println!("templates {}", template_paths().len());
        println!("commands {}", command_names().len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_fn_exists() {
        let _fn: fn(bool) -> lode_core::Result<()> = info;
    }
}
