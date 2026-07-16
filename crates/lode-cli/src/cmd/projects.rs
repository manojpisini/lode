#![deny(unsafe_code)]

use lode_core::{
    audit_project, load_global_config, load_registry, prune_registry, register_project,
    save_registry, LodeError,
};

use crate::{current_dir, ProjectsCommand};

pub(crate) fn projects(command: ProjectsCommand) -> lode_core::Result<()> {
    match command {
        ProjectsCommand::List { output, sort } => {
            let mut registry = load_registry()?;
            match sort.as_str() {
                "name" => registry
                    .projects
                    .sort_by(|left, right| left.name.cmp(&right.name)),
                "health" => registry
                    .projects
                    .sort_by(|left, right| left.path.exists().cmp(&right.path.exists()).reverse()),
                "last-seen" => registry
                    .projects
                    .sort_by(|left, right| right.last_seen.cmp(&left.last_seen)),
                other => {
                    return Err(LodeError::Message(format!(
                        "unsupported project sort: {other}"
                    )))
                }
            }
            if registry.projects.is_empty() {
                println!("no registered projects");
            } else if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&registry.projects)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                for project in registry.projects {
                    println!(
                        "{}\t{}\t{}\t{}",
                        project.name, project.profile, project.path, project.last_seen
                    );
                }
            }
        }
        ProjectsCommand::Cd { name } => {
            let registry = load_registry()?;
            let project = registry
                .projects
                .iter()
                .find(|project| project.name == name)
                .ok_or_else(|| LodeError::Message(format!("project not found: {name}")))?;
            println!("{}", project.path);
        }
        ProjectsCommand::Register { path } => {
            let path = path.unwrap_or(current_dir()?);
            let name = path
                .file_name()
                .map(str::to_string)
                .unwrap_or_else(|| "project".to_string());
            register_project(&name, &path, "manual")?;
            println!("registered {path}");
        }
        ProjectsCommand::Remove { name } => {
            let mut registry = load_registry()?;
            let before = registry.projects.len();
            registry.projects.retain(|project| project.name != name);
            let removed = before - registry.projects.len();
            if removed == 0 {
                return Err(LodeError::Message(format!("project not found: {name}")));
            }
            save_registry(&registry)?;
            println!("removed project {name}");
        }
        ProjectsCommand::Health {
            stale_only,
            output,
            refresh,
        } => {
            let registry = load_registry()?;
            let mut rows = Vec::new();
            for project in registry.projects {
                let status = if project.path.exists() {
                    "ok"
                } else {
                    "missing"
                };
                if stale_only && status == "ok" {
                    continue;
                }
                let score = if refresh && project.path.exists() {
                    audit_project(&project.path, &load_global_config()?)
                        .ok()
                        .map(|report| report.score)
                } else {
                    None
                };
                rows.push(serde_json::json!({
                    "name": project.name,
                    "status": status,
                    "path": project.path,
                    "score": score,
                }));
            }
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rows)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                for row in rows {
                    println!(
                        "{}\t{}\t{}\t{}",
                        row["name"].as_str().unwrap_or_default(),
                        row["status"].as_str().unwrap_or_default(),
                        row["score"]
                            .as_u64()
                            .map(|score| score.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                        row["path"].as_str().unwrap_or_default()
                    );
                }
            }
        }
        ProjectsCommand::Prune => {
            let removed = prune_registry()?;
            println!("project registry pruned: removed {removed}");
        }
    }
    Ok(())
}
