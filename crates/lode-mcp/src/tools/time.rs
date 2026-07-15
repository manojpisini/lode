use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

fn today_date_string() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let days = duration.as_secs() / 86400;
    let g = days as i64 + 719468;
    let year = (10000 * g + 14780) / 3652425;
    let doy = g - (365 * year + year / 4 - year / 100 + year / 400);
    let mi = (100 * doy + 52) / 3060;
    let month = (mi + 2) % 12 + 1;
    let year_out = year + (mi + 2) / 12;
    let day = doy - (mi * 306 + 5) / 10 + 1;
    format!("{year_out:04}-{month:02}-{day:02}")
}

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_time_today".to_string(),
            description: "Show today's time tracking summary".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_time_report".to_string(),
            description: "Show time tracking sessions".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "limit",
                    "Max number of sessions to show",
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

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root =
        camino::Utf8Path::from_path(validated.path()).ok_or_else(|| "non-utf8 path".to_string())?;
    let total_seconds = lode_core::time_today(root).map_err(|e| e.to_string())?;

    let log = lode_core::load_time_log(root).map_err(|e| e.to_string())?;
    let today = today_date_string();
    let today_sessions: Vec<_> = log
        .sessions
        .iter()
        .filter(|s| s.ended_at.starts_with(&today))
        .cloned()
        .collect();

    Ok(json!({
        "path": root.as_str(),
        "total_seconds": total_seconds,
        "sessions": today_sessions.iter().map(|s| json!({
            "started_at": s.started_at,
            "ended_at": s.ended_at,
            "seconds": s.seconds,
            "project": s.project,
            "file": s.file,
            "task": s.task,
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
        .unwrap_or(20)
        .min(1000);

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root =
        camino::Utf8Path::from_path(validated.path()).ok_or_else(|| "non-utf8 path".to_string())?;
    let log = lode_core::load_time_log(root).map_err(|e| e.to_string())?;

    let sessions: Vec<_> = log.sessions.iter().rev().take(limit).cloned().collect();

    Ok(json!({
        "path": root.as_str(),
        "total_sessions": log.sessions.len(),
        "showing": sessions.len(),
        "sessions": sessions.iter().map(|s| json!({
            "started_at": s.started_at,
            "ended_at": s.ended_at,
            "seconds": s.seconds,
            "project": s.project,
            "file": s.file,
            "task": s.task,
        })).collect::<Vec<_>>(),
    }))
}
