use serde_json::{json, Value};

use crate::schema::tool_input_schema;

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_projects_list".to_string(),
            description: "List all registered LODE projects".to_string(),
            input_schema: tool_input_schema(vec![]),
        },
        Tool {
            name: "lode_projects_health".to_string(),
            description: "Show health status for all registered projects".to_string(),
            input_schema: tool_input_schema(vec![]),
        },
    ]
}

pub fn lode_projects_list(_args: &Value) -> Result<Value, String> {
    let registry = lode_core::load_registry().map_err(|e| e.to_string())?;

    let projects: Vec<Value> = registry
        .projects
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "path": p.path.to_string(),
                "profile": p.profile,
                "last_seen": p.last_seen,
            })
        })
        .collect();

    Ok(json!({
        "total": projects.len(),
        "projects": projects,
    }))
}

pub fn lode_projects_health(_args: &Value) -> Result<Value, String> {
    let registry = lode_core::load_registry().map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for project in &registry.projects {
        let path = &project.path;
        let project_toml = path.join(".lode").join("project.toml");
        let healthy = project_toml.exists();
        results.push(json!({
            "name": project.name,
            "path": path.to_string(),
            "healthy": healthy,
        }));
    }

    Ok(json!({
        "total": results.len(),
        "projects": results,
    }))
}
