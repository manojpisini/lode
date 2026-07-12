use std::io::{self, BufRead, Write};

use serde_json::Value;

use crate::server::McpServer;

const MAX_MESSAGE_SIZE: usize = 1_048_576; // 1 MB

fn send_error(writer: &mut impl Write, code: i32, message: &str) {
    let error_response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": code, "message": message },
        "id": null,
    });
    if let Err(e) = writeln!(writer, "{error_response}") {
        eprintln!("lode-mcp: transport write error: {e}");
    }
    if let Err(e) = writer.flush() {
        eprintln!("lode-mcp: transport flush error: {e}");
    }
}

pub fn run_stdio_transport(server: &McpServer) {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(n) => {
                if n > MAX_MESSAGE_SIZE {
                    send_error(&mut writer, -32600, "Request too large");
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Error reading stdin: {e}");
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                send_error(&mut writer, -32700, &format!("Parse error: {e}"));
                continue;
            }
        };

        let response = server.handle_request(&request);

        if let Err(e) = writeln!(writer, "{response}") {
            eprintln!("lode-mcp: transport write error: {e}");
        }
        if let Err(e) = writer.flush() {
            eprintln!("lode-mcp: transport flush error: {e}");
        }
    }
}
