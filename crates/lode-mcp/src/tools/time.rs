use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_time_today".to_string(),
            description: "Show today's test history summary".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_time_report".to_string(),
            description: "Show test run history".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "limit",
                    "Max number of runs to show",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_time_today(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let root = std::path::Path::new(path);
    let history = lode_core::load_test_history(root).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "total_runs": history.runs.len(),
        "runs": history.runs.iter().map(|r| json!({
            "timestamp": r.timestamp,
            "passed": r.passed,
            "failed": r.failed,
            "duration_ms": r.duration_ms,
            "command": r.command,
        })).collect::<Vec<_>>(),
    }))
}

pub fn lode_time_report(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let limit = args["limit"]
        .as_str()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);

    let root = std::path::Path::new(path);
    let history = lode_core::load_test_history(root).map_err(|e| e.to_string())?;

    let runs: Vec<_> = history.runs.iter().rev().take(limit).collect();

    Ok(json!({
        "path": path,
        "total_runs": history.runs.len(),
        "showing": runs.len(),
        "runs": runs.iter().map(|r| json!({
            "timestamp": r.timestamp,
            "passed": r.passed,
            "failed": r.failed,
            "duration_ms": r.duration_ms,
            "command": r.command,
        })).collect::<Vec<_>>(),
    }))
}
