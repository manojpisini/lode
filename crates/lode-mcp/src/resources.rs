use serde_json::{json, Value};

pub fn list_resource_uris() -> Vec<String> {
    vec![
        "lode://config".to_string(),
        "lode://registry".to_string(),
        "lode://templates".to_string(),
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

pub fn read_resource(uri: &str) -> Result<Vec<Value>, String> {
    match uri {
        "lode://config" => read_config_resource(),
        "lode://registry" => read_registry_resource(),
        "lode://templates" => read_templates_resource(),
        "lode://profiles" => read_profiles_resource(),
        "lode://recipes" => read_recipes_resource(),
        "lode://project/info" => read_project_info(),
        "lode://project/health" => read_project_health(),
        "lode://project/metrics" => read_project_metrics(),
        _ => Err(format!("Unknown resource URI: {uri}")),
    }
}

fn read_config_resource() -> Result<Vec<Value>, String> {
    let config = lode_core::config::default_config();
    let content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://config",
        "mimeType": "application/toml",
        "text": content,
    })])
}

fn read_registry_resource() -> Result<Vec<Value>, String> {
    let registry = lode_core::load_registry().map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&registry).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://registry",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_templates_resource() -> Result<Vec<Value>, String> {
    let templates = lode_core::template_paths();
    let items: Vec<Value> = templates.iter().map(|name| json!({"name": name})).collect();
    let content = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://templates",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_profiles_resource() -> Result<Vec<Value>, String> {
    let profiles = lode_core::profile_names();
    let items: Vec<Value> = profiles.iter().map(|name| json!({"name": name})).collect();
    let content = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://profiles",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_recipes_resource() -> Result<Vec<Value>, String> {
    let recipes = lode_core::recipe_names();
    let items: Vec<Value> = recipes.iter().map(|name| json!({"name": name})).collect();
    let content = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://recipes",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_project_info() -> Result<Vec<Value>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let root = camino::Utf8PathBuf::from_path_buf(cwd).map_err(|_| "non-UTF-8 path".to_string())?;
    let project_toml = root.join(".lode").join("project.toml");

    if !project_toml.exists() {
        return Ok(vec![json!({
            "uri": "lode://project/info",
            "mimeType": "application/json",
            "text": "{}",
        })]);
    }

    let raw = std::fs::read_to_string(&project_toml).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://project/info",
        "mimeType": "application/toml",
        "text": raw,
    })])
}

fn read_project_health() -> Result<Vec<Value>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let root = camino::Utf8PathBuf::from_path_buf(cwd).map_err(|_| "non-UTF-8 path".to_string())?;

    let config = lode_core::config::default_config();
    let report = lode_core::audit_project(&root, &config).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://project/health",
        "mimeType": "application/json",
        "text": content,
    })])
}

fn read_project_metrics() -> Result<Vec<Value>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let root = camino::Utf8PathBuf::from_path_buf(cwd).map_err(|_| "non-UTF-8 path".to_string())?;

    let report = lode_core::load_metrics(&root).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    Ok(vec![json!({
        "uri": "lode://project/metrics",
        "mimeType": "application/json",
        "text": content,
    })])
}
