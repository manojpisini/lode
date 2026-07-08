use serde_json::{json, Value};

use crate::schema::{string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_sign".to_string(),
            description: "Compute content hash and show signature header for a file".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("file", "Relative file path to sign", string_schema()),
            ]),
        },
        Tool {
            name: "lode_stamp".to_string(),
            description: "Write a signature header into a file".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("file", "Relative file path to stamp", string_schema()),
            ]),
        },
    ]
}

pub fn lode_sign(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let file = args["file"]
        .as_str()
        .ok_or("Missing required argument: file")?;

    let root = camino::Utf8PathBuf::from(path);
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let file_path = validated.resolve(file).map_err(|e| e.to_string())?;

    if !file_path.exists() {
        return Err(format!("File not found: {file}"));
    }

    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let hash = lode_core::compute_content_hash(&content);
    let ext = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let comment_prefix = lode_core::comment_prefix_for_extension(ext)
        .unwrap_or("//")
        .to_string();
    let header = format!("{comment_prefix} lode:sha256={hash}");

    Ok(json!({
        "file": file,
        "hash": hash,
        "header": header,
        "has_signature": lode_core::has_signature_header(&content),
    }))
}

pub fn lode_stamp(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let file = args["file"]
        .as_str()
        .ok_or("Missing required argument: file")?;

    let root = camino::Utf8PathBuf::from(path);
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let file_path = validated.resolve(file).map_err(|e| e.to_string())?;

    if !file_path.exists() {
        return Err(format!("File not found: {file}"));
    }

    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let hash = lode_core::compute_content_hash(&content);
    let ext = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let comment_prefix = lode_core::comment_prefix_for_extension(ext)
        .unwrap_or("//")
        .to_string();
    let header = format!("{comment_prefix} lode:sha256={hash}\n");

    let new_content = format!("{header}{content}");
    validated
        .write_atomic(file, new_content)
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "file": file,
        "hash": hash,
    }))
}
