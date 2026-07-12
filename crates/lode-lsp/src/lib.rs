//! LSP (Language Server Protocol) server for LODE.
//!
//! Provides editor integration features: diagnostic reporting for secrets and
//! filename conventions, document symbols, completions, hover information,
//! and code actions.
#![deny(unsafe_code)]

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
    last_secret_scan: HashMap<String, (String, Vec<Value>)>,
    redact_diagnostics: bool,
}

const MAX_CONTENT_LENGTH: usize = 4 * 1024 * 1024; // 4 MB
const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "pub", "struct", "enum", "impl", "match", "if", "for", "while", "return", "use",
    "mod", "trait", "const", "static", "mut", "ref", "self", "super", "crate", "where", "as", "in",
    "type", "async", "await", "unsafe", "dyn", "move", "break", "continue", "else", "extern",
    "false", "true", "loop",
];

enum Action {
    Continue,
    Exit,
}

pub fn run() {
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
                    if let Some(len) = trimmed
                        .to_ascii_lowercase()
                        .strip_prefix("content-length: ")
                    {
                        content_length = len.trim().parse::<usize>().ok();
                    }
                }
                Err(_) => return,
            }
        }

        let len = content_length.unwrap_or(0);
        if len == 0 || len > MAX_CONTENT_LENGTH {
            continue;
        }

        let mut body = vec![0u8; len];
        if reader.read_exact(&mut body).is_err() {
            return;
        }

        let text = String::from_utf8_lossy(&body);
        if let Ok(msg) = serde_json::from_str::<JsonRpcMessage>(&text) {
            if let Some(method) = &msg.method {
                match handle_method(&mut state, &mut stdout, &msg, method) {
                    Ok(Action::Continue) => {}
                    Ok(Action::Exit) => return,
                    Err(e) => eprintln!("lode-lsp: handler error: {e}"),
                }
            }
        }
    }
}

fn handle_method(
    state: &mut LspState,
    stdout: &mut std::io::Stdout,
    msg: &JsonRpcMessage,
    method: &str,
) -> Result<Action, Box<dyn std::error::Error>> {
    match method {
        "initialize" => {
            let caps = serde_json::json!({
                "capabilities": {
                    "textDocumentSync": { "openClose": true, "change": 2, "save": { "includeText": true } },
                    "completionProvider": { "triggerCharacters": [".", "/"] },
                    "codeActionProvider": true,
                    "documentSymbolProvider": true,
                    "hoverProvider": true
                }
            });
            if let Some(params) = &msg.params {
                if let Some(options) = params.get("initializationOptions") {
                    if let Some(redact) = options.get("redactDiagnostics").and_then(|v| v.as_bool()) {
                        state.redact_diagnostics = redact;
                    }
                }
            }
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
        "exit" => return Ok(Action::Exit),
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
                    publish_diagnostics(stdout, state, uri, &text)?;
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
                    if let Some(text) = state.documents.get(uri).cloned() {
                        publish_diagnostics(stdout, state, uri, &text)?;
                    }
                }
            }
        }
        "textDocument/documentSymbol" => {
            let symbols = if let Some(params) = &msg.params {
                if let Some(uri) = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|u| u.as_str())
                {
                    if let Some(text) = state.documents.get(uri) {
                        document_symbols(uri, text)
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(serde_json::json!(symbols)),
                None,
            )?;
        }
        "textDocument/completion" => {
            let items = if let Some(params) = &msg.params {
                let uri = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                completion_items(uri)
            } else {
                completion_items("")
            };
            let result = serde_json::json!({
                "isIncomplete": false,
                "items": items
            });
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(result),
                None,
            )?;
        }
        "textDocument/hover" => {
            let hover = if let Some(params) = &msg.params {
                let uri = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                let line = params
                    .get("position")
                    .and_then(|p| p.get("line"))
                    .and_then(|l| l.as_u64())
                    .unwrap_or(0) as usize;
                let character = params
                    .get("position")
                    .and_then(|p| p.get("character"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0) as usize;
                hover_info(state, uri, line, character)
            } else {
                None
            };
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(serde_json::json!(hover)),
                None,
            )?;
        }
        "textDocument/codeAction" => {
            let uri = msg
                .params
                .as_ref()
                .and_then(|p| p.get("textDocument"))
                .and_then(|d| d.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let text = state.documents.get(uri);
            let secret_fix = text.map(|_| {
                serde_json::json!({
                    "title": "Add secrets to .env.example",
                    "kind": "quickfix",
                    "diagnostics": [],
                    "edit": { "changes": {} }
                })
            });
            let convention_fix = Some(serde_json::json!({
                "title": "Fix naming convention",
                "kind": "quickfix",
                "diagnostics": [],
                "edit": { "changes": {} }
            }));
            let rename_fix = Some(serde_json::json!({
                "title": "Rename file to match convention",
                "kind": "quickfix",
                "diagnostics": [],
                "edit": { "changes": {} }
            }));
            let mut actions = Vec::new();
            if let Some(action) = secret_fix {
                actions.push(action);
            }
            if let Some(action) = convention_fix {
                actions.push(action);
            }
            if let Some(action) = rename_fix {
                actions.push(action);
            }
            send_response(
                stdout,
                msg.id.clone().unwrap_or(Value::Null),
                Some(serde_json::json!(actions)),
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
    Ok(Action::Continue)
}

fn document_symbols(uri: &str, text: &str) -> Vec<Value> {
    let mut symbols = Vec::new();
    if uri.ends_with(".rs") {
        for (line_index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let (kind_char, name) = if let Some(name) = trimmed.strip_prefix("pub fn ") {
                ("Function", name)
            } else if let Some(name) = trimmed.strip_prefix("fn ") {
                ("Function", name)
            } else if let Some(name) = trimmed.strip_prefix("pub struct ") {
                ("Struct", name)
            } else if let Some(name) = trimmed.strip_prefix("struct ") {
                ("Struct", name)
            } else if let Some(name) = trimmed.strip_prefix("pub enum ") {
                ("Enum", name)
            } else if let Some(name) = trimmed.strip_prefix("enum ") {
                ("Enum", name)
            } else if let Some(name) = trimmed.strip_prefix("pub trait ") {
                ("Interface", name)
            } else if let Some(name) = trimmed.strip_prefix("trait ") {
                ("Interface", name)
            } else if trimmed.starts_with("impl") {
                let rest = trimmed.strip_prefix("impl").unwrap_or("").trim();
                let name = rest.split_whitespace().next().unwrap_or("impl");
                ("Method", name)
            } else if let Some(name) = trimmed.strip_prefix("pub mod ") {
                ("Module", name)
            } else if let Some(name) = trimmed.strip_prefix("mod ") {
                ("Module", name)
            } else {
                continue;
            };
            let symbol_name = name
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()
                .unwrap_or(name)
                .to_string();
            symbols.push(serde_json::json!({
                "name": symbol_name,
                "kind": kind_char,
                "location": {
                    "uri": uri,
                    "range": {
                        "start": { "line": line_index as u32, "character": 0 },
                        "end": { "line": line_index as u32, "character": line.len() as u32 }
                    }
                }
            }));
        }
    }
    symbols
}

fn completion_items(uri: &str) -> Vec<Value> {
    if uri.ends_with(".rs") {
        RUST_KEYWORDS
            .iter()
            .map(|kw| {
                serde_json::json!({
                    "label": kw,
                    "kind": 14,
                    "detail": "Rust keyword",
                    "insertText": kw
                })
            })
            .collect()
    } else if uri.ends_with(".toml") {
        vec![
            serde_json::json!({ "label": "schema_version", "detail": "Config schema version", "insertText": "schema_version = 3" }),
            serde_json::json!({ "label": "active_profile", "detail": "Active profile name", "insertText": "active_profile = \"\"" }),
            serde_json::json!({ "label": "[identity]", "detail": "Identity section" }),
            serde_json::json!({ "label": "[convention]", "detail": "Convention rules section" }),
            serde_json::json!({ "label": "[git]", "detail": "Git configuration section" }),
            serde_json::json!({ "label": "lode init", "detail": "Initialize project" }),
            serde_json::json!({ "label": "lode check", "detail": "Check conventions" }),
            serde_json::json!({ "label": "lode scan", "detail": "Scan secrets" }),
        ]
    } else {
        Vec::new()
    }
}

fn hover_info(state: &LspState, uri: &str, line: usize, character: usize) -> Option<Value> {
    let text = state.documents.get(uri)?;
    let line_content = text.lines().nth(line)?;
    let word = line_content
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .find(|w| {
            let start = line_content.find(w).unwrap_or(0);
            character >= start && character <= start + w.len()
        })?;
    if word.is_empty() {
        return None;
    }
    let detail = if RUST_KEYWORDS.contains(&word) {
        format!("Rust keyword: `{word}`")
    } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
        format!("Type or struct: `{word}`")
    } else if word.ends_with("()") || line_content.trim().starts_with("fn ") {
        format!("Function: `{word}`")
    } else {
        format!("Identifier: `{word}`")
    };
    Some(serde_json::json!({
        "contents": {
            "kind": "markdown",
            "value": detail
        }
    }))
}

fn publish_diagnostics(
    stdout: &mut std::io::Stdout,
    state: &mut LspState,
    uri: &str,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut diagnostics = Vec::new();

    // Use in-memory scan and cache results to avoid per-keystroke recomputation
    let secret_diagnostics =
        if state.last_secret_scan.get(uri).map(|(t, _)| t.as_str()) != Some(text) {
            let result = scan_secrets_via_core(text);
            state
                .last_secret_scan
                .insert(uri.to_string(), (text.to_string(), result.clone()));
            result
        } else {
            state
                .last_secret_scan
                .get(uri)
                .map(|(_, d)| d.clone())
                .unwrap_or_default()
        };
    diagnostics.extend(secret_diagnostics);

    // Redact diagnostic messages if configured
    if state.redact_diagnostics {
        for diag in &mut diagnostics {
            if let Some(msg) = diag.get("message").and_then(|m| m.as_str()) {
                let redacted = lode_core::redact(msg);
                if redacted != msg {
                    diag["message"] = serde_json::json!(redacted);
                }
            }
        }
    }

    // Delegate filename convention check to lode-core
    if let Some(convention_diag) = check_filename_convention(uri) {
        diagnostics.push(convention_diag);
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

fn scan_secrets_via_core(text: &str) -> Vec<Value> {
    let findings = lode_core::scan_content(text);
    findings
        .iter()
        .map(|finding| {
            let message = if finding.kind == "suspicious credential assignment" {
                "Potential secret detected: suspicious credential assignment".to_string()
            } else {
                format!("Potential secret detected: {}", finding.kind)
            };
            serde_json::json!({
                "range": {
                    "start": { "line": (finding.line - 1) as u32, "character": 0 },
                    "end": { "line": (finding.line - 1) as u32, "character": 0 }
                },
                "severity": 2,
                "message": message,
                "source": "lode"
            })
        })
        .collect()
}

fn check_filename_convention(uri: &str) -> Option<Value> {
    // Only accept file:// URIs to prevent injection of arbitrary path-like strings
    if !uri.starts_with("file://") {
        return None;
    }
    let filename = uri.rsplit('/').next().or_else(|| uri.rsplit('\\').next())?;
    let expected = lode_core::normalize_name(filename, &lode_core::default_config());
    if expected != filename {
        return Some(serde_json::json!({
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 1 }
            },
            "severity": 3,
            "message": format!("Filename '{filename}' should be '{expected}'"),
            "source": "lode"
        }));
    }
    None
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

#[cfg(test)]
mod lsp_unit_tests {
    use super::*;

    #[test]
    fn secret_diagnostics_use_core_scanner() {
        let diagnostics = scan_secrets_via_core("API_KEY=real-value\n");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn filename_diagnostics_use_core_normalization() {
        let diagnostic = check_filename_convention("file:///tmp/bad-name.rs").unwrap();
        assert!(diagnostic["message"]
            .as_str()
            .unwrap()
            .contains("bad_name.rs"));
        assert!(check_filename_convention("file:///tmp/README.md").is_none());
    }

    #[test]
    fn scan_secrets_returns_empty_for_clean_text() {
        let diagnostics = scan_secrets_via_core("fn main() { println!(\"hello\"); }\n");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn scan_secrets_returns_empty_for_empty_string() {
        let diagnostics = scan_secrets_via_core("");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn scan_secrets_detects_multiple_secrets() {
        let diagnostics = scan_secrets_via_core("API_KEY=secret123\nGITHUB_TOKEN=ghp_abc123\n");
        assert!(diagnostics.len() >= 2);
    }

    #[test]
    fn scan_secrets_diagnostics_have_expected_structure() {
        let diagnostics = scan_secrets_via_core("SECRET_KEY=my-secret\n");
        let diag = &diagnostics[0];
        assert!(diag.get("range").is_some());
        assert!(diag.get("message").is_some());
        assert!(diag.get("source").is_some());
        assert_eq!(diag["source"], "lode");
        assert!(diag.get("severity").is_some());
    }

    #[test]
    fn check_filename_convention_allows_snake_case() {
        let result = check_filename_convention("file:///project/my_file.rs");
        assert!(result.is_none());
    }

    #[test]
    fn check_filename_convention_flags_camel_case() {
        let result = check_filename_convention("file:///project/MyFile.rs");
        assert!(result.is_some());
    }

    #[test]
    fn check_filename_convention_handles_windows_paths() {
        let result = check_filename_convention("file:///C:/project/bad-name.rs");
        assert!(result.is_some());
        let result2 = check_filename_convention("file:///C:/project/README.md");
        assert!(result2.is_none());
    }

    #[test]
    fn check_filename_convention_diagnostic_has_correct_structure() {
        let diagnostic = check_filename_convention("file:///tmp/UPPER.rs").unwrap();
        assert!(diagnostic.get("range").is_some());
        assert!(diagnostic.get("message").is_some());
        assert!(diagnostic.get("source").is_some());
        assert_eq!(diagnostic["source"], "lode");
    }
}
