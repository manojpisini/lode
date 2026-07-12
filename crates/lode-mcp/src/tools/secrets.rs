use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "lode_scan_secrets".to_string(),
        description: "Scan project files for leaked secrets, API keys, and tokens".to_string(),
        input_schema: tool_input_schema(vec![
            ("path", "Project root directory", string_schema()),
            (
                "pattern",
                "Optional regex pattern to filter findings",
                optional_string_schema(),
            ),
        ]),
    }]
}

pub fn lode_scan_secrets(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = camino::Utf8PathBuf::from(path);

    let report = lode_core::scan_secrets(&root).map_err(|e| e.to_string())?;

    let findings: Vec<Value> = report
        .findings
        .iter()
        .map(|f| {
            json!({
                "file": f.path.to_string(),
                "line": f.line,
                "kind": f.kind,
            })
        })
        .collect();

    Ok(json!({
        "path": path,
        "checked_files": report.checked_files,
        "total_findings": report.findings.len(),
        "findings": findings,
        "status": if report.findings.is_empty() { "clean" } else { "findings" },
    }))
}
