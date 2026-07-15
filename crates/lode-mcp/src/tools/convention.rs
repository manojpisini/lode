use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};
use crate::util::load_config;

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_check".to_string(),
            description: "Check project for convention violations".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_fix".to_string(),
            description: "Automatically fix convention violations where possible".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_rename".to_string(),
            description: "Rename a file or directory to match conventions".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "target",
                    "Path to rename (relative to project root)",
                    string_schema(),
                ),
                (
                    "new_name",
                    "New name for the file/directory",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_check(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root =
        camino::Utf8Path::from_path(validated.path()).ok_or_else(|| "non-utf8 path".to_string())?;
    let config = load_config(root)?;

    let report = lode_core::check_path(root, &config).map_err(|e| e.to_string())?;

    let violations: Vec<Value> = report
        .violations
        .iter()
        .map(|v| {
            json!({
                "path": v.path.to_string(),
                "expected_name": v.expected_name,
            })
        })
        .collect();

    Ok(json!({
        "path": root.as_str(),
        "checked": report.checked,
        "violations_count": violations.len(),
        "violations": violations,
        "renamed": report.renamed.len(),
    }))
}

pub fn lode_fix(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root =
        camino::Utf8Path::from_path(validated.path()).ok_or_else(|| "non-utf8 path".to_string())?;
    let config = load_config(root)?;

    let report = lode_core::fix_path(root, &config).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": root.as_str(),
        "checked": report.checked,
        "remaining_violations": report.violations.len(),
        "renamed": report.renamed.len(),
        "renamed_files": report.renamed.iter().map(|(from, to)| {
            json!({"from": from.to_string(), "to": to.to_string()})
        }).collect::<Vec<_>>(),
    }))
}

pub fn lode_rename(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let target = args["target"]
        .as_str()
        .ok_or("Missing required argument: target")?;
    let new_name = args["new_name"].as_str().unwrap_or("");

    let root = camino::Utf8PathBuf::from(path);
    let config = load_config(&root)?;

    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let target_path = validated.resolve(target).map_err(|e| e.to_string())?;
    let relative_target = camino::Utf8Path::new(target);
    let parent = relative_target
        .parent()
        .unwrap_or(camino::Utf8Path::new(""));

    let name = if new_name.is_empty() {
        let stem = target_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Cannot determine file name")?;
        lode_core::normalize_name(stem, &config)
    } else {
        new_name.to_string()
    };

    let dest = parent.join(&name);
    validated
        .rename_entry(target, dest.as_str())
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "from": target.to_string(),
        "to": name,
    }))
}
