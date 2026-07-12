#![deny(unsafe_code)]

use camino::Utf8PathBuf;

use crate::{agent_sync, current_dir};
use crate::cmd::template::validate_template_tree;
use lode_core::{
    audit_project, global_asset_dir, load_global_config, save_metrics, sync_project, LodeError,
};

pub fn sync(dry_run: bool, force: bool, section: Option<&str>) -> lode_core::Result<()> {
    let sections = match section {
        Some(section) => vec![section.to_string()],
        None => vec![
            "config".to_string(),
            "templates".to_string(),
            "agent".to_string(),
            "metrics".to_string(),
        ],
    };
    for section in sections {
        match section.as_str() {
            "config" => {
                if dry_run {
                    println!("would sync config");
                    continue;
                }
                load_global_config()?;
                println!("synced config");
            }
            "templates" => {
                let cwd = current_dir()?;
                if cwd.join(".lode").join("project.toml").exists() {
                    let report = sync_project(cwd, load_global_config()?, force, dry_run)?;
                    if dry_run {
                        println!("would sync templates");
                        for path in report.planned_paths {
                            println!("would reconcile {path}");
                        }
                    } else {
                        println!(
                            "synced {} template-backed file(s)",
                            report.wrote_paths.len()
                        );
                    }
                } else if dry_run {
                    println!("would sync templates");
                } else {
                    validate_template_tree(&global_asset_dir("templates")?)?;
                    println!("synced templates");
                }
            }
            "agent" | "context" => {
                if dry_run {
                    println!("would sync {section}");
                    continue;
                }
                agent_sync()?;
            }
            "metrics" => {
                if dry_run {
                    println!("would sync metrics");
                    continue;
                }
                if force || Utf8PathBuf::from(".lode").exists() {
                    let cwd = current_dir()?;
                    let report = audit_project(&cwd, &load_global_config()?)?;
                    save_metrics(&cwd, &report)?;
                    println!("synced metrics");
                }
            }
            other => {
                return Err(LodeError::Message(format!(
                    "unsupported sync section: {other}"
                )))
            }
        }
    }
    Ok(())
}
