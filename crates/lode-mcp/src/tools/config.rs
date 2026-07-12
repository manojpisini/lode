use lode_core::ValidatedRoot;
use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_config_show".to_string(),
            description: "Show the default LODE configuration".to_string(),
            input_schema: tool_input_schema(vec![]),
        },
        Tool {
            name: "lode_config_set".to_string(),
            description: "Set a configuration value in the project's .lode/project.toml"
                .to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "key",
                    "Config key to set using dot notation",
                    string_schema(),
                ),
                ("value", "Value to set", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_config_validate".to_string(),
            description: "Validate a project configuration against the schema".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_config_show(_args: &Value) -> Result<Value, String> {
    let config = lode_core::config::default_config();
    let content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    Ok(json!({
        "config": content,
    }))
}

pub fn lode_config_set(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let key = args["key"]
        .as_str()
        .ok_or("Missing required argument: key")?;
    let value = args["value"].as_str().unwrap_or("");

    let root = ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;
    let raw = std::fs::read_to_string(root.path().join(".lode").join("project.toml"))
        .map_err(|_| format!("No LODE project found at {path}"))?;
    let mut config: toml::Value = toml::from_str(&raw).map_err(|e| e.to_string())?;

    let parts: Vec<&str> = key.split('.').collect();
    let mut current = &mut config;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(table) = current.as_table_mut() {
                table.insert(part.to_string(), toml::Value::String(value.to_string()));
            } else {
                return Err(format!("Cannot set key '{key}': parent is not a table"));
            }
        } else {
            current = current.get_mut(part).ok_or_else(|| {
                format!("Cannot set key '{key}': path segment '{part}' not found")
            })?;
        }
    }

    let new_content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    root.write_atomic(".lode/project.toml", &new_content)
        .map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(json!({
        "status": "ok",
        "key": key,
        "value": value,
        "config": new_content,
    }))
}

pub fn lode_config_validate(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let root = ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;
    let project_toml = root.path().join(".lode").join("project.toml");

    if !project_toml.exists() {
        return Err(format!("No LODE project found at {path}"));
    }

    let raw = std::fs::read_to_string(&project_toml).map_err(|e| e.to_string())?;
    let _config: lode_core::config::LodeConfig = toml::from_str(&raw).map_err(|e| e.to_string())?;

    Ok(json!({
        "valid": true,
        "path": path,
    }))
}
