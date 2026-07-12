#![deny(unsafe_code)]

use lode_core::{add_component_to_project, load_global_config, AddRequest};

pub fn add_component(component: &str, dry_run: bool, overwrite: bool) -> lode_core::Result<()> {
    let cwd = crate::current_dir()?;
    let project_name = cwd
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "project".to_string());
    let report = add_component_to_project(AddRequest {
        name: project_name,
        project_dir: cwd,
        config: load_global_config()?,
        component: component.to_string(),
        dry_run,
        overwrite,
    })?;
    for path in if dry_run {
        report.planned_paths
    } else {
        report.wrote_paths
    } {
        println!("{} {}", if dry_run { "would add" } else { "added" }, path);
    }
    Ok(())
}
