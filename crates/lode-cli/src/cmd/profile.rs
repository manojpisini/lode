#![deny(unsafe_code)]

use lode_core::{
    global_asset_dir, load_global_config, profile_names, save_global_config, LodeError,
    ValidatedRoot,
};

use crate::{LibraryCommand, ProfileCommand};

pub(crate) fn profile_command(command: ProfileCommand) -> lode_core::Result<()> {
    match command {
        ProfileCommand::List => {
            for profile in profile_names() {
                println!("{profile}");
            }
        }
        ProfileCommand::Show { name, output } => {
            if output.should_use_json() {
                let profiles_dir = global_asset_dir("profiles")?;
                let path = profiles_dir.join(format!("{name}.toml"));
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| LodeError::Message(format!("profile not found: {name}: {e}")))?;
                let value: serde_json::Value = toml::from_str(&content)
                    .map_err(|e| LodeError::Message(format!("failed to parse profile: {e}")))?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .map_err(|e| LodeError::Message(e.to_string()))?
                );
                return Ok(());
            }
            crate::cmd::template::library_command(
                "profiles",
                LibraryCommand::Show { name, raw: true },
                &profile_names(),
            )?;
        }
        ProfileCommand::Use { name } => {
            let mut config = load_global_config()?;
            config.active_profile = Some(name.clone());
            save_global_config(&config)?;
            println!("active profile: {name}");
        }
        ProfileCommand::New { name } => {
            let root = ValidatedRoot::new(global_asset_dir("profiles")?)?;
            let relative = format!("{name}.toml");
            let config = load_global_config()?;
            let raw = toml::to_string_pretty(&config)?;
            root.write_atomic(relative, raw)?;
            println!("created profile {name}");
        }
        ProfileCommand::Delete { name } => {
            if profile_names().iter().any(|profile| *profile == name) {
                return Err(LodeError::Message(format!(
                    "refusing to delete embedded profile: {name}"
                )));
            }
            let root = ValidatedRoot::new(global_asset_dir("profiles")?)?;
            root.remove_file(format!("{name}.toml"))?;
            println!("deleted profile {name}");
        }
    }
    Ok(())
}
