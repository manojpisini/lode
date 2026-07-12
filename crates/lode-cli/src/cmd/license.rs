#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::{global_asset_dir, ValidatedRoot, LodeError};

use std::fs;

use crate::{
    add_license, collect_file_names, current_dir, license_path, list_dir, project_license_id,
    read_license, safe_relative_path, LicenseCommand,
};
use lode_core::load_global_config;

pub(crate) fn license(command: LicenseCommand) -> lode_core::Result<()> {
    match command {
        LicenseCommand::List { format } => {
            let root = global_asset_dir("licenses")?;
            if format == "json" {
                let mut items = Vec::new();
                collect_file_names(&root, &mut items)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                list_dir(root)?;
            }
        }
        LicenseCommand::Show { id } => print!("{}", read_license(&id)?),
        LicenseCommand::Info { id } => {
            let contents = read_license(&id)?;
            println!("id: {id}");
            println!("bytes: {}", contents.len());
            println!(
                "category: {}",
                if id.contains(" OR ") {
                    "compound"
                } else {
                    "single"
                }
            );
        }
        LicenseCommand::Add { id, file, text } => {
            add_license(&id, file, text.as_deref())?;
        }
        LicenseCommand::Remove { id } => {
            let path = license_path(&id)?;
            if !path.exists() {
                return Err(LodeError::Message(format!("license not found: {id}")));
            }
            let root = ValidatedRoot::new(global_asset_dir("licenses")?)?;
            root.remove_file(safe_relative_path(&format!("{id}.txt"))?)?;
            println!("removed license {id}");
        }
        LicenseCommand::Set { id } => {
            let contents = read_license(&id)?;
            ValidatedRoot::new(current_dir()?)?.write_atomic("LICENSE", contents)?;
            println!("license set: {id}");
        }
        LicenseCommand::Check { json } => {
            let path = Utf8PathBuf::from("LICENSE");
            let ok = path.exists()
                && !fs::read_to_string(&path)
                    .map_err(|source| LodeError::Io {
                        path: path.as_str().into(),
                        source,
                    })?
                    .trim()
                    .is_empty();
            if json {
                println!("{{\"license\":{ok}}}");
            } else if ok {
                println!("license ok");
            } else {
                return Err(LodeError::Message(
                    "LICENSE is missing or empty".to_string(),
                ));
            }
        }
        LicenseCommand::Apply { dry_run } => {
            let id = project_license_id()?.unwrap_or(load_global_config()?.identity.license);
            if dry_run {
                println!("would apply license {id}");
            } else {
                let contents = read_license(&id)?;
                ValidatedRoot::new(current_dir()?)?.write_atomic("LICENSE", contents)?;
                println!("license applied: {id}");
            }
        }
    }
    Ok(())
}
