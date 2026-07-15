use serde_json::{json, Value};

use crate::error::McpError;

pub fn list_resource_uris() -> Vec<String> {
    vec![
        "lode://config".to_string(),
        "lode://registry".to_string(),
        "lode://templates".to_string(),
        "lode://template-bundles".to_string(),
        "lode://profiles".to_string(),
        "lode://recipes".to_string(),
        "lode://project/info".to_string(),
        "lode://project/health".to_string(),
        "lode://project/metrics".to_string(),
    ]
}

pub fn list_resources() -> Vec<Value> {
    vec![
        json!({
            "uri": "lode://config",
            "name": "LODE Config",
            "description": "The default LODE configuration template",
            "mimeType": "application/toml",
        }),
        json!({
            "uri": "lode://registry",
            "name": "Project Registry",
            "description": "All registered LODE projects",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://templates",
            "name": "Templates",
            "description": "Available project template paths",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://template-bundles",
            "name": "Template Bundles",
            "description": "Available template bundles in the global templates directory",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://profiles",
            "name": "Profiles",
            "description": "Available scaffold profiles",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://recipes",
            "name": "Recipes",
            "description": "Available component recipes",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://project/info",
            "name": "Project Info",
            "description": "Current project configuration and metadata",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://project/health",
            "name": "Project Health",
            "description": "Current project health audit",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "lode://project/metrics",
            "name": "Project Metrics",
            "description": "Current project metrics",
            "mimeType": "application/json",
        }),
    ]
}

type ResResult<T> = Result<T, McpError>;

pub fn read_resource(uri: &str) -> ResResult<Vec<Value>> {
    match uri {
        "lode://config" => read_config_resource(),
        "lode://registry" => read_registry_resource(),
        "lode://templates" => read_templates_resource(),
        "lode://template-bundles" => read_template_bundles_resource(),
        "lode://profiles" => read_profiles_resource(),
        "lode://recipes" => read_recipes_resource(),
        "lode://project/info" => read_project_info(),
        "lode://project/health" => read_project_health(),
        "lode://project/metrics" => read_project_metrics(),
        _ => Err(McpError::NotFound(format!("Unknown resource URI: {uri}"))),
    }
}

fn read_config_resource() -> ResResult<Vec<Value>> {
    let config = lode_core::config::default_config();
    let content = toml::to_string_pretty(&config)?;
    Ok(vec![json!({
        "uri": "lode://config",
        "mimeType": "application/toml",
        "text": content,
    })])
}

fn read_registry_resource() -> ResResult<Vec<Value>> {
    let registry = lode_core::load_registry()?;
    let content = serde_json::to_string_pretty(&registry)?;
    Ok(vec![json!({
        "uri": "lode://registry",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_templates_resource() -> ResResult<Vec<Value>> {
    let templates = lode_core::template_paths();
    let items: Vec<Value> = templates.iter().map(|name| json!({"name": name})).collect();
    let content = serde_json::to_string_pretty(&items)?;
    Ok(vec![json!({
        "uri": "lode://templates",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_template_bundles_resource() -> ResResult<Vec<Value>> {
    let search_dir = lode_core::global_dir()
        .map(|g| g.into_std_path_buf().join("templates"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

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
                let manifest_path = p.join(format!("{dirname}.toml"));
                if manifest_path.exists() {
                    bundles.push(json!({
                        "name": dirname,
                        "path": p.to_string_lossy(),
                    }));
                }
            }
        }
    }

    let content = serde_json::to_string_pretty(&bundles)?;
    Ok(vec![json!({
        "uri": "lode://template-bundles",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_profiles_resource() -> ResResult<Vec<Value>> {
    let profiles = lode_core::profile_names();
    let items: Vec<Value> = profiles.iter().map(|name| json!({"name": name})).collect();
    let content = serde_json::to_string_pretty(&items)?;
    Ok(vec![json!({
        "uri": "lode://profiles",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_recipes_resource() -> ResResult<Vec<Value>> {
    let recipes = lode_core::recipe_names();
    let items: Vec<Value> = recipes.iter().map(|name| json!({"name": name})).collect();
    let content = serde_json::to_string_pretty(&items)?;
    Ok(vec![json!({
        "uri": "lode://recipes",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_project_info() -> ResResult<Vec<Value>> {
    let cwd = std::env::current_dir()?;
    let root = camino::Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| McpError::Internal("non-UTF-8 path".to_string()))?;
    let project_toml = root.join(".lode").join("project.toml");

    if !project_toml.exists() {
        return Ok(vec![json!({
            "uri": "lode://project/info",
            "mimeType": "application/json",
            "text": "{}",
        })]);
    }

    let raw = std::fs::read_to_string(&project_toml)?;
    Ok(vec![json!({
        "uri": "lode://project/info",
        "mimeType": "application/toml",
        "text": raw,
    })])
}

fn read_project_health() -> ResResult<Vec<Value>> {
    let cwd = std::env::current_dir()?;
    let root = camino::Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| McpError::Internal("non-UTF-8 path".to_string()))?;

    let config = lode_core::config::default_config();
    let report = lode_core::audit_project(&root, &config)?;
    let content = serde_json::to_string_pretty(&report)?;
    Ok(vec![json!({
        "uri": "lode://project/health",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_project_metrics() -> ResResult<Vec<Value>> {
    let cwd = std::env::current_dir()?;
    let root = camino::Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| McpError::Internal("non-UTF-8 path".to_string()))?;

    let report = lode_core::load_metrics(&root)?;
    let content = serde_json::to_string_pretty(&report)?;
    Ok(vec![json!({
        "uri": "lode://project/metrics",
        "mimeType": "application/json",
        "text": content,
    })])
}
