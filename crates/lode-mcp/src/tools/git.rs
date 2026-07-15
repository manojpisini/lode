use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_git_branch".to_string(),
            description: "Generate a conventional branch name from a description".to_string(),
            input_schema: tool_input_schema(vec![
                (
                    "kind",
                    "Branch kind (feat, fix, chore, etc.)",
                    string_schema(),
                ),
                ("description", "Branch description", string_schema()),
            ]),
        },
        Tool {
            name: "lode_git_commit".to_string(),
            description: "Stage all changes and create a conventional commit".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("message", "Commit message", string_schema()),
            ]),
        },
        Tool {
            name: "lode_git_changelog".to_string(),
            description: "Generate a changelog from git log".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("from_tag", "Start from this tag", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_git_tag".to_string(),
            description: "Create a git tag for the current HEAD".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("tag", "Tag name (e.g. v1.0.0)", string_schema()),
                (
                    "message",
                    "Tag message (optional)",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_git_branch(args: &Value) -> Result<Value, String> {
    let kind = args["kind"]
        .as_str()
        .ok_or("Missing required argument: kind")?;
    let description = args["description"]
        .as_str()
        .ok_or("Missing required argument: description")?;

    let branch = lode_core::branch_name(kind, description);

    Ok(json!({
        "branch": branch,
    }))
}

pub fn lode_git_commit(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let message = args["message"]
        .as_str()
        .ok_or("Missing required argument: message")?;

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = validated.path();

    if !lode_core::is_git_repo(root) {
        return Err("Not a git repository".to_string());
    }

    lode_core::git_add_all(root).map_err(|e| e.to_string())?;
    lode_core::git_commit(root, message).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "message": message,
    }))
}

pub fn lode_git_changelog(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let from_tag = args["from_tag"].as_str();

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = validated.path();

    if !lode_core::is_git_repo(root) {
        return Err("Not a git repository".to_string());
    }

    let changelog = lode_core::git_changelog(root, from_tag).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": root.display().to_string(),
        "changelog": changelog,
    }))
}

pub fn lode_git_tag(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let tag = args["tag"]
        .as_str()
        .ok_or("Missing required argument: tag")?;
    let message = args["message"].as_str();

    let validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = validated.path();

    if !lode_core::is_git_repo(root) {
        return Err("Not a git repository".to_string());
    }

    lode_core::git_tag(root, tag, message).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "tag": tag,
    }))
}
