#![deny(unsafe_code)]

use crate::SelfCommand;
use lode_core::{global_dir, ValidatedRoot};
use std::env;

pub(crate) fn self_command(command: SelfCommand) -> lode_core::Result<()> {
    match command {
        SelfCommand::Info => {
            let exe = env::current_exe().map_err(|source| lode_core::LodeError::Io {
                path: "current_exe".into(),
                source,
            })?;
            let root = global_dir()?;
            println!("version\t{}", env!("CARGO_PKG_VERSION"));
            println!("executable\t{}", exe.display());
            println!("global_dir\t{root}");
            println!("schema_version\t3");
            for name in [
                "templates",
                "profiles",
                "snippets",
                "licenses",
                "recipes",
                "plugins",
                "commands",
            ] {
                println!("{name}\t{}", crate::count_dir_entries(&root.join(name))?);
            }
            println!(
                "upgrade_cache\t{}",
                crate::count_dir_entries(&root.join("cache").join("upgrade"))?
            );
        }
        SelfCommand::Clean { dry_run } => {
            let root_path = global_dir()?;
            let root = ValidatedRoot::new(&root_path)?;
            for path in crate::self_clean_targets()? {
                if dry_run {
                    println!("would clean {path}");
                } else if path.exists() {
                    let relative = path.strip_prefix(&root_path).map_err(|_| {
                        lode_core::LodeError::Message(format!(
                            "clean target is outside global root: {path}"
                        ))
                    })?;
                    if path.is_dir() {
                        root.remove_dir_all(relative)?;
                    } else {
                        root.remove_file(relative)?;
                    }
                    println!("cleaned {path}");
                }
            }
        }
        SelfCommand::Uninstall { keep_config } => {
            let root_path = global_dir()?;
            if keep_config {
                let root = ValidatedRoot::new(&root_path)?;
                for name in [
                    "cache",
                    "logs",
                    "templates",
                    "profiles",
                    "snippets",
                    "licenses",
                    "recipes",
                    "commands",
                ] {
                    let path = root_path.join(name);
                    if path.exists() {
                        root.remove_dir_all(name)?;
                    }
                }
                println!("removed generated Lode data; kept config.toml");
            } else if root_path.exists() {
                let parent = root_path.parent().ok_or_else(|| {
                    lode_core::LodeError::Message(format!(
                        "global root has no parent: {root_path}"
                    ))
                })?;
                let name = root_path.file_name().ok_or_else(|| {
                    lode_core::LodeError::Message(format!(
                        "global root has no directory name: {root_path}"
                    ))
                })?;
                ValidatedRoot::new(parent)?.remove_dir_all(name)?;
                println!("removed {root_path}");
            }
        }
    }
    Ok(())
}
