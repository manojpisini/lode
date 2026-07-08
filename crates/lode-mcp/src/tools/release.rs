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
    let _bump = args["bump"]
        .as_str()
        .ok_or("Missing required argument: bump")?;
    let dry_run = args["dry_run"].as_bool().unwrap_or(false);

    let root = std::path::Path::new(path);
    let config = lode_core::ReleaseConfig::default();

    match lode_core::create_release(root, &config, dry_run) {
        Ok(report) => Ok(json!({
            "status": "ok",
            "path": path,
            "old_version": report.old_version,
            "new_version": report.new_version,
            "tag": report.tag,
            "files_updated": report.files_updated.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
            "dry_run": report.dry_run,
        })),
        Err(e) => Err(e.to_string()),
    }
}
