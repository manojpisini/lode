#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fs;

use camino::Utf8PathBuf;
use lode_core::{
    command_names, default_lodepack_checksum_algorithm, global_asset_dir, LodeError, LodePack,
    LodePackFile, LodePackManifest, ValidatedRoot,
};

use crate::{
    content_hash_bytes, current_dir, list_dir, now_timestamp, open_editor, resolve_command_path,
    run_command_macro_loaded, safe_relative_path, CommandsCommand, OutputFormat,
};

pub(crate) fn commands(command: CommandsCommand) -> lode_core::Result<()> {
    match command {
        CommandsCommand::List => {
            for command in command_names() {
                println!("{command}");
            }
            list_dir(Utf8PathBuf::from(".lode").join("commands"))?;
        }
        CommandsCommand::Show { name, output } => {
            let path = resolve_command_path(&name)?;
            let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            if output.should_use_json() {
                let value: serde_json::Value = toml::from_str(&raw)
                    .map_err(|e| LodeError::Message(format!("failed to parse command: {e}")))?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .map_err(|e| LodeError::Message(e.to_string()))?
                );
            } else {
                print!("{raw}");
            }
        }
        CommandsCommand::Add { slug, global, from } => {
            add_command_macro(&slug, global, from.as_deref())?;
        }
        CommandsCommand::Remove { slug, global } => {
            remove_command_macro(&slug, global)?;
        }
        CommandsCommand::Export { out } => {
            export_command_macros(out)?;
        }
        CommandsCommand::Run { slug, dry_run } => {
            let path = resolve_command_path(&slug)?;
            let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let value: toml::Value =
                toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
            run_command_macro_loaded(&slug, &value, &HashMap::new(), dry_run)?
        }
        CommandsCommand::Edit { name } => {
            let path = resolve_command_path(&name)?;
            open_editor(&path)?;
        }
    }
    Ok(())
}

fn add_command_macro(slug: &str, global: bool, from: Option<&str>) -> lode_core::Result<()> {
    let path = command_macro_path(slug, global)?;
    if path.exists() {
        return Err(LodeError::Message(format!(
            "command macro already exists: {slug}"
        )));
    }
    let root_path = if global {
        global_asset_dir("commands")?
    } else {
        current_dir()?
    };
    let root = ValidatedRoot::new(&root_path)?;
    let relative = if global {
        safe_relative_path(&format!("{slug}.toml"))?
    } else {
        Utf8PathBuf::from(".lode")
            .join("commands")
            .join(safe_relative_path(&format!("{slug}.toml"))?)
    };
    if let Some(parent) = relative.parent() {
        root.create_dir_all(parent)?;
    }
    let contents = if let Some(source_slug) = from {
        let source = resolve_command_path(source_slug)?;
        fs::read_to_string(&source).map_err(|source_error| LodeError::Io {
            path: source.as_str().into(),
            source: source_error,
        })?
    } else {
        format!(
            "slug = \"{slug}\"\ndescription = \"Custom {slug} command macro\"\n\n[[steps]]\nkind = \"make\"\nrun = \"{slug}\"\n"
        )
    };
    root.write_atomic(relative, contents)?;
    println!("created command macro {slug} at {path}");
    Ok(())
}

fn remove_command_macro(slug: &str, global: bool) -> lode_core::Result<()> {
    let path = command_macro_path(slug, global)?;
    if !path.exists() {
        return Err(LodeError::Message(format!(
            "command macro not found: {slug}"
        )));
    }
    let root_path = if global {
        global_asset_dir("commands")?
    } else {
        current_dir()?
    };
    let root = ValidatedRoot::new(&root_path)?;
    let relative = if global {
        safe_relative_path(&format!("{slug}.toml"))?
    } else {
        Utf8PathBuf::from(".lode")
            .join("commands")
            .join(safe_relative_path(&format!("{slug}.toml"))?)
    };
    root.remove_file(relative)?;
    println!("removed command macro {slug}");
    Ok(())
}

fn export_command_macros(out: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    let mut pack = LodePack {
        version: 1,
        manifest: LodePackManifest {
            schema_version: 3,
            lode_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now_timestamp(),
            file_count: 0,
            checksum_algorithm: default_lodepack_checksum_algorithm(),
        },
        files: Vec::new(),
    };
    let global = global_asset_dir("commands")?;
    collect_command_macro_files(&global, "global", &mut pack)?;
    let local = Utf8PathBuf::from(".lode").join("commands");
    collect_command_macro_files(&local, "project", &mut pack)?;
    pack.manifest.file_count = pack.files.len();
    let raw = serde_json::to_string_pretty(&pack)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    if let Some(path) = out {
        let project_dir = current_dir()?;
        let root = ValidatedRoot::new(&project_dir)?;
        let relative = if path.is_absolute() {
            path.strip_prefix(&project_dir).map_err(|_| {
                LodeError::Message(format!("export path is outside the project root: {path}"))
            })?
        } else {
            path.as_path()
        };
        if let Some(parent) = relative.parent() {
            root.create_dir_all(parent)?;
        }
        root.write_atomic(relative, &raw)?;
        println!("exported {} command macros to {path}", pack.files.len());
    } else {
        println!("{raw}");
    }
    Ok(())
}

fn collect_command_macro_files(
    root: &Utf8PathBuf,
    prefix: &str,
    pack: &mut LodePack,
) -> lode_core::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|source| LodeError::Io {
        path: root.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: root.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if path.extension() != Some("toml") {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        let name = path.file_name().unwrap_or("command.toml");
        let checksum = content_hash_bytes(contents.as_bytes());
        pack.files.push(LodePackFile {
            path: format!("{prefix}/commands/{name}"),
            contents,
            checksum,
        });
    }
    Ok(())
}

fn command_macro_path(slug: &str, global: bool) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&format!("{slug}.toml"))?;
    if global {
        Ok(global_asset_dir("commands")?.join(relative))
    } else {
        Ok(Utf8PathBuf::from(".lode").join("commands").join(relative))
    }
}
