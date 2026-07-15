use serde_json::{json, Value};

use crate::schema::{bool_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "lode_release".to_string(),
        description: "Bump version and prepare a release".to_string(),
        input_schema: tool_input_schema(vec![
            ("path", "Project root directory", string_schema()),
            ("bump", "Bump type: major, minor, or patch", string_schema()),
            ("dry_run", "Preview without making changes", bool_schema()),
        ]),
    }]
}

pub fn lode_release(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let bump = args["bump"]
        .as_str()
        .ok_or("Missing required argument: bump")?;
    if !["major", "minor", "patch"].contains(&bump) {
        return Err(format!(
            "Invalid bump type '{bump}'. Must be one of: major, minor, patch"
        ));
    }
    let dry_run = args["dry_run"].as_bool().unwrap_or(false);

    let root = validated.path();
    let config = lode_core::ReleaseConfig::default();

    match lode_core::create_release(root, &config, dry_run) {
        Ok(report) => Ok(json!({
            "status": "ok",
            "path": root.display().to_string(),
            "old_version": report.old_version,
            "new_version": report.new_version,
            "tag": report.tag,
            "files_updated": report.files_updated.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
            "dry_run": report.dry_run,
        })),
        Err(e) => Err(e.to_string()),
    }
}
