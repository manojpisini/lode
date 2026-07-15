use serde_json::{json, Value};

use crate::schema::string_schema;

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_agent_sync".to_string(),
            description: "Show agent configuration sync status for a project".to_string(),
            input_schema: crate::schema::tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_agent_plan".to_string(),
            description: "Generate an execution plan for a task".to_string(),
            input_schema: crate::schema::tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("task", "Task description", string_schema()),
            ]),
        },
    ]
}

pub fn lode_agent_sync(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = camino::Utf8PathBuf::from_path_buf(validated.path().to_path_buf())
        .map_err(|_| "non-utf8 path".to_string())?;

    let agents_dir = root.join(".lode").join("agents");
    let exists = agents_dir.exists();

    Ok(json!({
        "path": root.to_string(),
        "agents_dir": agents_dir.to_string(),
        "exists": exists,
        "status": "ok",
    }))
}

pub fn lode_agent_plan(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let task = args["task"]
        .as_str()
        .ok_or("Missing required argument: task")?;

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let mut steps = Vec::new();
    steps.push(json!({"step": 1, "action": "analyse", "description": format!("Analyse project at {}", validated.path().display())}));
    steps.push(
        json!({"step": 2, "action": "plan", "description": format!("Plan execution for: {task}")}),
    );
    steps.push(json!({"step": 3, "action": "execute", "description": "Execute planned steps"}));
    steps.push(json!({"step": 4, "action": "verify", "description": "Verify results"}));

    Ok(json!({
        "path": validated.path().display().to_string(),
        "task": task,
        "steps": steps,
    }))
}
