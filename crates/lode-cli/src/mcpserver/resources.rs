use serde_json::{json, Value};

use super::error::McpError;

type ResResult<T> = Result<T, McpError>;

pub fn list_resources() -> Vec<Value> {
    vec![
        json!({"uri": "lode://config", "name": "LODE Config", "description": "The default LODE configuration template", "mimeType": "application/toml"}),
        json!({"uri": "lode://registry", "name": "Project Registry", "description": "All registered LODE projects", "mimeType": "application/json"}),
        json!({"uri": "lode://templates", "name": "Templates", "description": "Available project template paths", "mimeType": "application/json"}),
        json!({"uri": "lode://template-bundles", "name": "Template Bundles", "description": "Available template bundles in the global templates directory", "mimeType": "application/json"}),
        json!({"uri": "lode://profiles", "name": "Profiles", "description": "Available scaffold profiles", "mimeType": "application/json"}),
        json!({"uri": "lode://recipes", "name": "Recipes", "description": "Available component recipes", "mimeType": "application/json"}),
        json!({"uri": "lode://project/info", "name": "Project Info", "description": "Current project configuration and metadata", "mimeType": "application/json"}),
        json!({"uri": "lode://project/health", "name": "Project Health", "description": "Current project health audit", "mimeType": "application/json"}),
        json!({"uri": "lode://project/metrics", "name": "Project Metrics", "description": "Current project metrics", "mimeType": "application/json"}),
    ]
}

pub fn read_resource(uri: &str) -> ResResult<Vec<Value>> {
    match uri {
        "lode://config" => read_config(),
        "lode://registry" => read_registry(),
        "lode://templates" => read_templates(),
        "lode://template-bundles" => read_template_bundles(),
        "lode://profiles" => read_profiles(),
        "lode://recipes" => read_recipes(),
        "lode://project/info" => read_project_info(),
        "lode://project/health" => read_project_health(),
        "lode://project/metrics" => read_project_metrics(),
        _ => Err(McpError::NotFound(format!("Unknown resource URI: {uri}"))),
    }
}

fn read_config() -> ResResult<Vec<Value>> {
    let content = toml::to_string_pretty(&lode_core::config::default_config())?;
    Ok(vec![
        json!({"uri": "lode://config", "mimeType": "application/toml", "text": content}),
    ])
}

fn read_registry() -> ResResult<Vec<Value>> {
    let content = serde_json::to_string_pretty(&lode_core::load_registry()?)?;
    Ok(vec![
        json!({"uri": "lode://registry", "mimeType": "application/json", "text": content}),
    ])
}

fn read_templates() -> ResResult<Vec<Value>> {
    let items: Vec<Value> = lode_core::template_paths()
        .iter()
        .map(|n| json!({"name": n}))
        .collect();
    let content = serde_json::to_string_pretty(&items)?;
    Ok(vec![
        json!({"uri": "lode://templates", "mimeType": "application/json", "text": content}),
    ])
}

fn read_template_bundles() -> ResResult<Vec<Value>> {
    let search_dir = lode_core::global_dir()
        .ok()
        .map(|g| g.into_std_path_buf().join("templates"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let mut bundles = Vec::new();
    if search_dir.exists() {
        for entry in
            std::fs::read_dir(&search_dir).map_err(|e| McpError::Internal(e.to_string()))?
        {
            let entry = entry.map_err(|e| McpError::Internal(e.to_string()))?;
            let p = entry.path();
            if p.is_dir() {
                let dirname = p
                    .file_name()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default()
                    .to_string();
                if p.join(format!("{dirname}.toml")).exists() {
                    bundles.push(json!({"name": dirname, "path": p.to_string_lossy()}));
                }
            }
        }
    }
    Ok(vec![
        json!({"uri": "lode://template-bundles", "mimeType": "application/json", "text": serde_json::to_string_pretty(&bundles)?}),
    ])
}

fn read_profiles() -> ResResult<Vec<Value>> {
    let items: Vec<Value> = lode_core::profile_names()
        .iter()
        .map(|n| json!({"name": n}))
        .collect();
    let content = serde_json::to_string_pretty(&items)?;
    Ok(vec![
        json!({"uri": "lode://profiles", "mimeType": "application/json", "text": content}),
    ])
}

fn read_recipes() -> ResResult<Vec<Value>> {
    let items: Vec<Value> = lode_core::recipe_names()
        .iter()
        .map(|n| json!({"name": n}))
        .collect();
    let content = serde_json::to_string_pretty(&items)?;
    Ok(vec![
        json!({"uri": "lode://recipes", "mimeType": "application/json", "text": content}),
    ])
}

fn read_project_info() -> ResResult<Vec<Value>> {
    let root = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|_| McpError::Internal("non-UTF-8 path".to_string()))?;
    let p = root.join(".lode").join("project.toml");
    if !p.exists() {
        return Ok(vec![
            json!({"uri": "lode://project/info", "mimeType": "application/json", "text": "{}"}),
        ]);
    }
    let raw = std::fs::read_to_string(&p)?;
    Ok(vec![
        json!({"uri": "lode://project/info", "mimeType": "application/toml", "text": raw}),
    ])
}

fn read_project_health() -> ResResult<Vec<Value>> {
    let root = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|_| McpError::Internal("non-UTF-8 path".to_string()))?;
    let report = lode_core::audit_project(&root, &lode_core::config::default_config())?;
    Ok(vec![
        json!({"uri": "lode://project/health", "mimeType": "application/json", "text": serde_json::to_string_pretty(&report)?}),
    ])
}

fn read_project_metrics() -> ResResult<Vec<Value>> {
    let root = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|_| McpError::Internal("non-UTF-8 path".to_string()))?;
    let report = lode_core::load_metrics(&root)?;
    Ok(vec![
        json!({"uri": "lode://project/metrics", "mimeType": "application/json", "text": serde_json::to_string_pretty(&report)?}),
    ])
}
