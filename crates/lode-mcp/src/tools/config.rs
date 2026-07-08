use serde_json::{json, Value};

use crate::schema::{string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_config_show".to_string(),
            description: "Show the default LODE configuration".to_string(),
            input_schema: tool_input_schema(vec![]),
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
    let _key = args["key"]
        .as_str()
        .ok_or("Missing required argument: key")?;
    let _value = args["value"].as_str().unwrap_or("");

    Err("config_set is not yet implemented for MCP".to_string())
}

pub fn lode_config_validate(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let root = camino::Utf8PathBuf::from(path);
    let project_toml = root.join(".lode").join("project.toml");

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
