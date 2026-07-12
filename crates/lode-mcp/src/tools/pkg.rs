use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_pkg_outdated".to_string(),
            description: "List outdated dependencies".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_pkg_audit".to_string(),
            description: "Audit dependencies for known vulnerabilities".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_pkg_update".to_string(),
            description: "Update dependencies".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "package",
                    "Package name to update (optional)",
                    optional_string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_pkg_list".to_string(),
            description: "Detect the package manager for a project".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_pkg_clean".to_string(),
            description: "Show clean command for detected package manager".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_pkg_outdated(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = std::path::Path::new(path);
    let pm = lode_core::detect_package_manager(root).ok_or("No package manager detected")?;

    let args = lode_core::package_outdated_args(&pm).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "package_manager": pm,
        "args": args,
    }))
}

pub fn lode_pkg_audit(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = std::path::Path::new(path);
    let pm = lode_core::detect_package_manager(root).ok_or("No package manager detected")?;

    let args = lode_core::package_audit_args(&pm, None).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "package_manager": pm,
        "args": args,
    }))
}

pub fn lode_pkg_update(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let package = args["package"].as_str();

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = std::path::Path::new(path);
    let pm = lode_core::detect_package_manager(root).ok_or("No package manager detected")?;

    let args = lode_core::package_update_args(&pm, package).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "package_manager": pm,
        "args": args,
    }))
}

pub fn lode_pkg_list(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = std::path::Path::new(path);
    let pm = lode_core::detect_package_manager(root);

    Ok(json!({
        "path": path,
        "package_manager": pm,
    }))
}

pub fn lode_pkg_clean(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = std::path::Path::new(path);
    let pm = lode_core::detect_package_manager(root).ok_or("No package manager detected")?;

    Ok(json!({
        "path": path,
        "package_manager": pm,
        "command": format!("{pm} clean"),
    }))
}
