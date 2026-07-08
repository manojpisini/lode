use std::collections::HashMap;
use std::io::{BufRead, Read, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct JsonRpcMessage {
    #[serde(rename = "jsonrpc")]
    _jsonrpc: String,
    id: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcNotification {
    jsonrpc: &'static str,
    method: String,
    params: Value,
}

#[derive(Default)]
struct LspState {
    documents: HashMap<String, String>,
}

pub async fn run() {
    let mut state = LspState::default();
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();

    loop {
        let mut content_length: Option<usize> = None;

        loop {
            let mut header = String::new();
            match reader.read_line(&mut header) {
                Ok(0) => return,
                Ok(_) => {
                    let trimmed = header.trim();
                    if trimmed.is_empty() {
                        break;
                    }
                    if let Some(len) = trimmed.strip_prefix("Content-Length: ") {
                        content_length = len.trim().parse::<usize>().ok();
                    }
                }
                Err(_) => return,
            }
        }

        let len = content_length.unwrap_or(0);
        if len == 0 {
            continue;
        }

        let mut body = vec![0u8; len];
        if reader.read_exact(&mut body).is_err() {
            return;
        }

        let text = String::from_utf8_lossy(&body);
        if let Ok(msg) = serde_json::from_str::<JsonRpcMessage>(&text) {
            if let Some(method) = &msg.method {
                handle_method(&mut state, &mut stdout, &msg, method).ok();
            }
        }
    }
}

fn handle_method(
    state: &mut LspState,
    stdout: &mut std::io::Stdout,
    msg: &JsonRpcMessage,
    method: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match method {
        "initialize" => {
            let caps = serde_json::json!({
                "capabilities": {
                    "textDocumentSync": { "openClose": true, "change": 2, "save": { "includeText": true } },
                    "completionProvider": { "triggerCharacters": [".", "/"] },
                    "codeActionProvider": true
                }
            });
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(caps),
                None,
            )?;
        }
        "initialized" => {}
        "shutdown" => {
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(Value::Null),
                None,
            )?;
        }
        "exit" => std::process::exit(0),
        "textDocument/didOpen" | "textDocument/didChange" => {
            if let Some(params) = &msg.params {
                if let Some(uri) = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|u| u.as_str())
                {
                    let text = if method == "textDocument/didOpen" {
                        params
                            .get("textDocument")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string()
                    } else {
                        params
                            .get("contentChanges")
                            .and_then(|c| c.as_array())
                            .and_then(|changes| changes.last())
                            .and_then(|last| last.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string()
                    };
                    state.documents.insert(uri.to_string(), text.clone());
                    publish_diagnostics(stdout, uri, &text)?;
                }
            }
        }
        "textDocument/didSave" => {
            if let Some(params) = &msg.params {
                if let Some(uri) = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|u| u.as_str())
                {
                    if let Some(text) = state.documents.get(uri) {
                        publish_diagnostics(stdout, uri, text)?;
                    }
                }
            }
        }
        "textDocument/completion" => {
            let items = serde_json::json!({
                "isIncomplete": false,
                "items": [
                    { "label": "schema_version", "detail": "Config schema version", "insertText": "schema_version = 3" },
                    { "label": "active_profile", "detail": "Active profile name", "insertText": "active_profile = \"\"" },
                    { "label": "[identity]", "detail": "Identity section" },
                    { "label": "[convention]", "detail": "Convention rules section" },
                    { "label": "[git]", "detail": "Git configuration section" },
                    { "label": "lode init", "detail": "Initialize project" },
                    { "label": "lode check", "detail": "Check conventions" },
                    { "label": "lode scan", "detail": "Scan secrets" },
                ]
            });
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(items),
                None,
            )?;
        }
        "textDocument/codeAction" => {
            let actions = serde_json::json!([{
                "title": "Fix naming convention",
                "kind": "quickfix",
                "diagnostics": [],
                "edit": { "changes": {} }
            }]);
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(actions),
                None,
            )?;
        }
        _ => {
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                None,
                Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {method}"),
                }),
            )?;
        }
    }
    Ok(())
}

fn publish_diagnostics(
    stdout: &mut std::io::Stdout,
    uri: &str,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut diagnostics = Vec::new();

    for (i, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        if (lower.contains("password")
            || lower.contains("secret")
            || lower.contains("api_key")
            || lower.contains("token"))
            && (line.contains('=') || line.contains(':'))
        {
            diagnostics.push(serde_json::json!({
                "range": {
                    "start": { "line": i as u32, "character": 0 },
                    "end": { "line": i as u32, "character": line.len() as u32 }
                },
                "severity": 2,
                "message": "Potential secret detected",
                "source": "lode"
            }));
        }
    }

    if let Some(filename) = uri.rsplit('/').next().or_else(|| uri.rsplit('\\').next()) {
        let stem = filename
            .trim_end_matches(".rs")
            .trim_end_matches(".py")
            .trim_end_matches(".ts");
        if stem.contains('-') {
            diagnostics.push(serde_json::json!({
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 0, "character": 1 }
                },
                "severity": 3,
                "message": format!("Filename '{filename}' uses hyphens; prefer snake_case"),
                "source": "lode"
            }));
        }
    }

    let notification = JsonRpcNotification {
        jsonrpc: "2.0",
        method: "textDocument/publishDiagnostics".to_string(),
        params: serde_json::json!({
            "uri": uri,
            "diagnostics": diagnostics
        }),
    };

    let body = serde_json::to_string(&notification)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stdout.write_all(header.as_bytes())?;
    stdout.write_all(body.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn send_response(
    stdout: &mut std::io::Stdout,
    id: Value,
    result: Option<Value>,
    error: Option<JsonRpcError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result,
        error,
    };
    let body = serde_json::to_string(&response)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stdout.write_all(header.as_bytes())?;
    stdout.write_all(body.as_bytes())?;
    stdout.flush()?;
    Ok(())
}
