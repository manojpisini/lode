use std::io::{self, BufRead, Write};

use serde_json::Value;

use crate::server::McpServer;

#[derive(Debug, Clone)]
pub enum Transport {
    Stdio,
    Http { port: u16, host: String },
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
            Ok(_) => {}
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
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {e}")
                    },
                    "id": null,
                });
                writeln!(writer, "{error_response}").ok();
                writer.flush().ok();
                continue;
            }
        };

        let response = server.handle_request(&request);

        writeln!(writer, "{response}").ok();
        writer.flush().ok();
    }
}

pub fn start_transport(_transport: Transport, server: &McpServer) {
    match _transport {
        Transport::Stdio => run_stdio_transport(server),
        Transport::Http { port: _, host: _ } => {
            eprintln!("HTTP transport not yet implemented");
        }
    }
}
