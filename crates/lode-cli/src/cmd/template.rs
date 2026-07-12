#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{global_asset_dir, template_paths, LodeError, ValidatedRoot};

use crate::LibraryCommand;

pub(crate) fn library_command(
    root: &str,
    command: LibraryCommand,
    embedded: &[&str],
) -> lode_core::Result<()> {
    match command {
        LibraryCommand::List { format } => {
            if format == "json" {
                println!(
                    "{}",
                    serde_json::to_string_pretty(embedded)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                for item in embedded {
                    println!("{item}");
                }
            }
        }
        LibraryCommand::Show { name, raw: _ } => {
            let mut path = global_asset_dir(root)?.join(&name);
            if !path.exists() && matches!(root, "profiles" | "commands" | "recipes") {
                path = global_asset_dir(root)?.join(format!("{name}.toml"));
            }
            if path.exists() {
                print!(
                    "{}",
                    fs::read_to_string(&path).map_err(|source| LodeError::Io {
                        path: path.as_str().into(),
                        source,
                    })?
                );
            } else if embedded.iter().any(|item| *item == name) {
                println!("{name}");
            } else {
                return Err(LodeError::Message(format!("{root} item not found: {name}")));
            }
        }
        LibraryCommand::Diff { name } => {
            require_template_library(root)?;
            let relative = crate::safe_relative_path(&name)?;
            let path = global_asset_dir(root)?.join(&relative);
            let current = fs::read_to_string(&path).unwrap_or_default();
            let default = embedded_template(&name)?;
            if current == default {
                println!("template unchanged: {name}");
            } else {
                println!("template differs: {name}");
                crate::print_simple_diff(&current, &default);
            }
        }
        LibraryCommand::Reset { name } => {
            require_template_library(root)?;
            let relative = crate::safe_relative_path(&name)?;
            let asset_dir = global_asset_dir(root)?;
            let validated_root = ValidatedRoot::new(&asset_dir)?;
            let contents = embedded_template(&name)?;
            if let Some(parent) = relative.parent() {
                validated_root.create_dir_all(parent)?;
            }
            validated_root.write_atomic(&relative, contents)?;
            println!("reset template {name}");
        }
        LibraryCommand::Validate { all } => {
            require_template_library(root)?;
            if all {
                for item in embedded {
                    validate_template(item)?;
                }
                println!("validated {} templates", embedded.len());
            } else {
                let root = global_asset_dir(root)?;
                validate_template_tree(&root)?;
                println!("templates valid");
            }
        }
        LibraryCommand::Edit { name } => {
            let asset_dir = global_asset_dir(root)?;
            let path = asset_dir.join(&name);
            if !path.exists() && matches!(root, "profiles" | "commands" | "recipes") {
                let path_with_ext = asset_dir.join(format!("{name}.toml"));
                if path_with_ext.exists() {
                    crate::open_editor(&path_with_ext)?;
                } else {
                    return Err(LodeError::Message(format!("{root} item not found: {name}")));
                }
            } else if path.exists() {
                crate::open_editor(&path)?;
            } else if embedded.iter().any(|item| *item == name) {
                println!("{name} is a built-in template; use `{root} reset {name}` to make a local copy first");
            } else {
                return Err(LodeError::Message(format!("{root} item not found: {name}")));
            }
        }
    }
    Ok(())
}

fn require_template_library(root: &str) -> lode_core::Result<()> {
    if root == "templates" {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "{root} does not support this library operation"
        )))
    }
}

fn embedded_template(name: &str) -> lode_core::Result<String> {
    if !template_paths().contains(&name) {
        return Err(LodeError::Message(format!("template not found: {name}")));
    }
    let context = lode_core::RenderContext::new()
        .with("project", "project")
        .with("project_ident", "project")
        .with("project_class", "Project")
        .with("author", "Your Name")
        .with("org", "namespace")
        .with("license", "MIT OR Apache-2.0")
        .with("year", lode_core::current_year())
        .with("profile", "core/bare");
    Ok(lode_core::assets::template_contents(name, &context))
}

pub(crate) fn validate_template_tree(root: &Utf8PathBuf) -> lode_core::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    if root.is_dir() {
        for entry in fs::read_dir(root).map_err(|source| LodeError::Io {
            path: root.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: root.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            validate_template_tree(&child)?;
        }
    } else {
        validate_template(root.as_str())?;
    }
    Ok(())
}

fn validate_template(name: &str) -> lode_core::Result<()> {
    let contents = if Utf8PathBuf::from(name).exists() {
        fs::read_to_string(name).map_err(|source| LodeError::Io {
            path: name.into(),
            source,
        })?
    } else {
        embedded_template(name).unwrap_or_default()
    };
    if name.ends_with(".toml") {
        let _: toml::Value =
            toml::from_str(&contents).map_err(|error| LodeError::Message(error.to_string()))?;
    } else if name.ends_with(".json") {
        let _: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|error| LodeError::Message(error.to_string()))?;
    } else if contents.trim().is_empty() {
        return Err(LodeError::Message(format!("empty template: {name}")));
    }
    Ok(())
}
