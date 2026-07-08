use serde_json::{json, Value};

use crate::schema::tool_input_schema;

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_audit".to_string(),
            description: "Run a project health audit (conventions, secrets, files)".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                crate::schema::string_schema(),
            )]),
        },
        Tool {
            name: "lode_metrics".to_string(),
            description: "Show project metrics (audit report from .lode/metrics.json)".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                crate::schema::string_schema(),
            )]),
        },
    ]
}

pub fn lode_audit(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let root = camino::Utf8PathBuf::from(path);

    if !root.join(".lode").exists() {
        return Err(format!("No LODE project found at {path}"));
    }

    let config = lode_core::config::default_config();
    let report = lode_core::audit_project(&root, &config).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "score": report.score,
        "convention_violations": report.convention_violations,
        "secret_findings": report.secret_findings,
        "license_present": report.license_present,
        "env_example_present": report.env_example_present,
        "readme_present": report.readme_present,
    }))
}

pub fn lode_metrics(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let root = camino::Utf8PathBuf::from(path);

    let report = lode_core::load_metrics(&root).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "score": report.score,
        "convention_violations": report.convention_violations,
        "secret_findings": report.secret_findings,
        "license_present": report.license_present,
        "env_example_present": report.env_example_present,
        "readme_present": report.readme_present,
    }))
}
