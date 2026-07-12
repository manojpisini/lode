use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_env_check".to_string(),
            description: "Check environment variables for drift or missing values".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_env_add".to_string(),
            description: "Add a new environment variable to the .env config".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("key", "Environment variable name", string_schema()),
                ("value", "Default value", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_env_sync".to_string(),
            description: "Synchronise .env file with the env config".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_env_check(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = camino::Utf8PathBuf::from(path);
    let env_config = lode_core::EnvConfig::default();

    let drifts =
        lode_core::check_env_drift(root.as_std_path(), &env_config).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "drift_count": drifts.len(),
        "drifts": drifts.iter().map(|d| json!({
            "key": d.key,
            "issue": d.issue,
        })).collect::<Vec<_>>(),
        "status": if drifts.is_empty() { "ok" } else { "drift" },
    }))
}

pub fn lode_env_add(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let key = args["key"]
        .as_str()
        .ok_or("Missing required argument: key")?;

    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "Invalid key '{key}': must contain only alphanumeric characters and underscores"
        ));
    }

    let value = args["value"].as_str().unwrap_or("");

    if value.contains('\n') || value.contains('\r') || value.contains('\0') {
        return Err("Invalid .env value: must not contain newlines or null bytes".to_string());
    }

    let root = camino::Utf8PathBuf::from(path);
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let env_path = validated.resolve(".env").map_err(|e| e.to_string())?;

    let entry = format!("{key}={value}\n");
    if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path).map_err(|e| e.to_string())?;
        let new_content = format!("{existing}{entry}");
        validated
            .write_atomic(".env", new_content)
            .map_err(|e| e.to_string())?;
    } else {
        validated
            .write_atomic(".env", entry)
            .map_err(|e| e.to_string())?;
    }

    Ok(json!({
        "status": "ok",
        "key": key,
        "added": true,
    }))
}

pub fn lode_env_sync(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = camino::Utf8PathBuf::from(path);
    let project_name = root.file_name().unwrap_or("project");
    let env_config = lode_core::EnvConfig::default();

    lode_core::generate_env(root.as_std_path(), &env_config, project_name)
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "path": path,
    }))
}
