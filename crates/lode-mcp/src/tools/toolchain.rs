use serde_json::{json, Value};

use crate::schema::{string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_toolchain_status".to_string(),
            description: "Show installed toolchain versions for a project".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_toolchain_pin".to_string(),
            description: "Pin a specific tool version in the toolchain store".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "runtime",
                    "Runtime name (e.g. rust, node, python, go)",
                    string_schema(),
                ),
                ("version", "Version to pin", string_schema()),
            ]),
        },
    ]
}

pub fn lode_toolchain_status(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let root = std::path::Path::new(path);
    let config = lode_core::ToolchainConfig::default();

    let statuses = lode_core::toolchain_status(root, &config);

    let tools: Vec<Value> = statuses
        .iter()
        .map(|s| {
            json!({
                "runtime": s.runtime,
                "installed": s.installed,
                "version": s.version,
                "lock_version": s.lock_version,
                "manager": s.manager,
            })
        })
        .collect();

    Ok(json!({
        "path": path,
        "tools": tools,
    }))
}

pub fn lode_toolchain_pin(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let runtime_name = args["runtime"]
        .as_str()
        .ok_or("Missing required argument: runtime")?;
    let version = args["version"]
        .as_str()
        .ok_or("Missing required argument: version")?;

    let root = std::path::Path::new(path);
    let config = lode_core::ToolchainConfig::default();

    let runtime = config
        .runtimes
        .iter()
        .find(|r| r.name == runtime_name)
        .ok_or_else(|| format!("Unknown runtime: {runtime_name}"))?;

    lode_core::pin_runtime(root, runtime, version).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "runtime": runtime_name,
        "version": version,
    }))
}
