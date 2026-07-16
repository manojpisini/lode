use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use lode_core::ipc::{port_path, socket_port};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

pub struct DaemonControl {
    pub shutdown_tx: mpsc::Sender<()>,
    pub paused: Arc<AtomicBool>,
}

impl DaemonControl {
    pub fn new(shutdown_tx: mpsc::Sender<()>) -> Self {
        Self {
            shutdown_tx,
            paused: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("IPC server failed: {0}")]
    ServerFailed(String),
    #[error("Command parse error: {0}")]
    ParseError(String),
    #[error("Authentication failed")]
    AuthError,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcCommand {
    Status,
    Stop,
    Pause,
    Resume,
    ListWatchers,
    Reload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    command: String,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl IpcResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

pub struct IpcServer {
    socket_path: PathBuf,
    running: bool,
    auth_token: String,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        let auth_token = generate_token();
        Self {
            socket_path,
            running: false,
            auth_token,
        }
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    pub async fn start(&mut self) -> Result<(), IpcError> {
        if self.running {
            return Err(IpcError::ServerFailed("Already running".to_string()));
        }

        if cfg!(unix) && self.socket_path.exists() {
            tokio::fs::remove_file(&self.socket_path).await?;
        }
        if cfg!(not(unix)) {
            let p = port_path(&self.socket_path);
            if p.exists() {
                tokio::fs::remove_file(&p).await?;
            }
        }

        // Write auth token to sidecar file
        let token_path = token_path(&self.socket_path);
        tokio::fs::write(&token_path, &self.auth_token).await?;

        self.running = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), IpcError> {
        if !self.running {
            return Err(IpcError::ServerFailed("Not running".to_string()));
        }

        if self.socket_path.exists() {
            tokio::fs::remove_file(&self.socket_path).await?;
        }
        let p = port_path(&self.socket_path);
        if p.exists() {
            tokio::fs::remove_file(&p).await?;
        }
        let t = token_path(&self.socket_path);
        if t.exists() {
            tokio::fs::remove_file(&t).await?;
        }

        self.running = false;
        Ok(())
    }
}

fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    // Deterministic token based on time + pid (non-cryptographic, sufficient for local IPC)
    let raw = format!("lode-ipc-{pid}-{nanos}");
    let hash = simple_hash(&raw);
    format!("lode-ipc-token-{pid}-{hash}")
}

fn simple_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn token_path(socket_path: &Path) -> PathBuf {
    let mut p = socket_path.to_path_buf();
    p.set_extension("token");
    p
}

pub fn read_token(socket_path: &Path) -> Result<String, IpcError> {
    let token_path = token_path(socket_path);
    std::fs::read_to_string(&token_path)
        .map(|s| s.trim().to_string())
        .map_err(|e| IpcError::ServerFailed(format!("read token {}: {e}", token_path.display())))
}

fn authenticate(token: &str, expected: &str) -> Result<(), IpcError> {
    if token == expected {
        Ok(())
    } else {
        Err(IpcError::AuthError)
    }
}

pub fn handle_command(command: &IpcCommand, control: &DaemonControl) -> IpcResponse {
    match command {
        IpcCommand::Status => {
            let paused = control.paused.load(Ordering::SeqCst);
            IpcResponse::ok("Daemon is running")
                .with_data(serde_json::json!({"status": if paused { "paused" } else { "running" }}))
        }
        IpcCommand::Stop => {
            let _ = control.shutdown_tx.try_send(());
            IpcResponse::ok("Stop requested")
        }
        IpcCommand::Pause => {
            control.paused.store(true, Ordering::SeqCst);
            IpcResponse::ok("Paused")
        }
        IpcCommand::Resume => {
            control.paused.store(false, Ordering::SeqCst);
            IpcResponse::ok("Resumed")
        }
        IpcCommand::ListWatchers => {
            IpcResponse::ok("Watchers listed").with_data(serde_json::json!({"watchers": []}))
        }
        IpcCommand::Reload => IpcResponse::ok("Configuration reloaded"),
    }
}

pub fn parse_json_request(input: &str) -> Result<IpcRequest, IpcError> {
    serde_json::from_str::<IpcRequest>(input)
        .map_err(|e| IpcError::ParseError(format!("Invalid JSON request: {e}")))
}

pub fn parse_command(input: &str) -> Result<IpcCommand, IpcError> {
    match input.trim().to_lowercase().as_str() {
        "status" => Ok(IpcCommand::Status),
        "stop" => Ok(IpcCommand::Stop),
        "pause" => Ok(IpcCommand::Pause),
        "resume" => Ok(IpcCommand::Resume),
        "list" | "list-watchers" => Ok(IpcCommand::ListWatchers),
        "reload" => Ok(IpcCommand::Reload),
        other => Err(IpcError::ParseError(format!("Unknown command: {other}"))),
    }
}

fn process_ipc_line(line: &str, auth_token: &str, control: &DaemonControl) -> Option<IpcResponse> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let (command_str, token_str) = if line.starts_with('{') {
        match parse_json_request(line) {
            Ok(req) => (req.command.clone(), req.token.unwrap_or_default()),
            Err(_) => return Some(IpcResponse::error("Invalid JSON request format")),
        }
    } else {
        let mut parts = line.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").to_string();
        let tok = parts.next().unwrap_or("").to_string();
        (cmd, tok)
    };

    if authenticate(&token_str, auth_token).is_err() {
        return Some(IpcResponse::error("Authentication failed: invalid token"));
    }

    match parse_command(&command_str) {
        Ok(command) => Some(handle_command(&command, control)),
        Err(e) => Some(IpcResponse::error(e.to_string())),
    }
}

#[cfg(unix)]
pub async fn run_ipc_listener(
    socket_path: PathBuf,
    auth_token: String,
    control: DaemonControl,
) -> Result<(), IpcError> {
    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }

    let listener = tokio::net::UnixListener::bind(&socket_path)?;
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("lode-daemon: accept error: {e}");
                continue;
            }
        };
        let token = auth_token.clone();
        let ctrl = DaemonControl {
            shutdown_tx: control.shutdown_tx.clone(),
            paused: Arc::clone(&control.paused),
        };
        tokio::spawn(async move {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some(response) = process_ipc_line(&line, &token, &ctrl) {
                            if let Ok(json) = serde_json::to_string(&response) {
                                let mut inner = reader.get_mut();
                                if let Err(e) = inner.write_all(json.as_bytes()).await {
                                    eprintln!("lode-daemon: ipc write error: {e}");
                                }
                                if let Err(e) = inner.flush().await {
                                    eprintln!("lode-daemon: ipc flush error: {e}");
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

fn try_claim_lock(lock_path: &Path) -> Result<std::fs::File, IpcError> {
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(lock_path)
        .map_err(|e| IpcError::ServerFailed(format!("lock {}: {e}", lock_path.display())))
}

#[cfg(not(unix))]
pub async fn run_ipc_listener(
    socket_path: PathBuf,
    auth_token: String,
    control: DaemonControl,
) -> Result<(), IpcError> {
    use tokio::net::TcpListener;

    // Claim lock atomically before bind to prevent TOCTOU race
    let lock_path = port_path(&socket_path).with_extension("lock");
    let _lock = try_claim_lock(&lock_path)?;
    tokio::fs::write(&lock_path, std::process::id().to_string()).await?;

    let base_port = socket_port(&socket_path);
    let mut port = base_port;
    let listener = loop {
        let addr = format!("127.0.0.1:{port}");
        match TcpListener::bind(&addr).await {
            Ok(l) => break l,
            Err(_) if port < base_port + 1000 => {
                port += 1;
                continue;
            }
            Err(e) => return Err(IpcError::ServerFailed(e.to_string())),
        }
    };

    // Write the port file before the accept loop so clients can discover the port.
    let port_file = port_path(&socket_path);
    tokio::fs::write(&port_file, port.to_string()).await?;

    eprintln!("lode-daemon: IPC listening on 127.0.0.1:{port}");

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("lode-daemon: tcp accept error: {e}");
                continue;
            }
        };
        let token = auth_token.clone();
        let ctrl = DaemonControl {
            shutdown_tx: control.shutdown_tx.clone(),
            paused: Arc::clone(&control.paused),
        };
        tokio::spawn(async move {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some(response) = process_ipc_line(&line, &token, &ctrl) {
                            if let Ok(json) = serde_json::to_string(&response) {
                                let inner = reader.get_mut();
                                if let Err(e) = inner.write_all(json.as_bytes()).await {
                                    eprintln!("lode-daemon: ipc write error: {e}");
                                }
                                if let Err(e) = inner.flush().await {
                                    eprintln!("lode-daemon: ipc flush error: {e}");
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

#[cfg(test)]
mod daemon_ipc_tests {
    use super::*;

    #[test]
    fn parses_supported_commands() {
        assert_eq!(parse_command("status").unwrap(), IpcCommand::Status);
        assert_eq!(
            parse_command("list-watchers").unwrap(),
            IpcCommand::ListWatchers
        );
        assert!(parse_command("missing").is_err());
    }

    #[test]
    fn status_response_contains_running_state() {
        let (tx, _rx) = mpsc::channel(1);
        let control = DaemonControl::new(tx);
        let response = handle_command(&IpcCommand::Status, &control);
        assert!(response.success);
        assert_eq!(response.data.unwrap()["status"], "running");
    }

    #[test]
    fn tcp_fallback_port_is_stable() {
        let path = std::path::Path::new(".lode/daemon/daemon.sock");
        assert_eq!(socket_port(path), socket_port(path));
    }

    #[test]
    fn tcp_fallback_port_in_valid_range() {
        let path = std::path::Path::new(".lode/daemon/daemon.sock");
        let port = socket_port(path);
        assert!(
            (42000..62000).contains(&port),
            "port {port} out of range 42000-61999"
        );
    }

    #[test]
    fn tcp_fallback_port_differs_for_different_paths() {
        let p1 = socket_port(std::path::Path::new("project-a/daemon.sock"));
        let p2 = socket_port(std::path::Path::new("project-b/daemon.sock"));
        // Extremely unlikely to collide
        assert_ne!(p1, p2);
    }

    #[test]
    fn parse_all_known_commands() {
        assert_eq!(parse_command("status").unwrap(), IpcCommand::Status);
        assert_eq!(parse_command("stop").unwrap(), IpcCommand::Stop);
        assert_eq!(parse_command("pause").unwrap(), IpcCommand::Pause);
        assert_eq!(parse_command("resume").unwrap(), IpcCommand::Resume);
        assert_eq!(parse_command("list").unwrap(), IpcCommand::ListWatchers);
        assert_eq!(
            parse_command("list-watchers").unwrap(),
            IpcCommand::ListWatchers
        );
        assert_eq!(parse_command("reload").unwrap(), IpcCommand::Reload);
    }

    #[test]
    fn parse_case_insensitive_commands() {
        assert_eq!(parse_command("Status").unwrap(), IpcCommand::Status);
        assert_eq!(parse_command("STOP").unwrap(), IpcCommand::Stop);
    }

    #[test]
    fn parse_unknown_command_returns_error() {
        assert!(parse_command("unknown").is_err());
        assert!(parse_command("").is_err());
    }

    #[test]
    fn handle_all_commands_return_success() {
        let (tx, _rx) = mpsc::channel(1);
        let control = DaemonControl::new(tx);
        for cmd in &[
            IpcCommand::Status,
            IpcCommand::Stop,
            IpcCommand::Pause,
            IpcCommand::Resume,
            IpcCommand::ListWatchers,
            IpcCommand::Reload,
        ] {
            let response = handle_command(cmd, &control);
            assert!(response.success, "command {cmd:?} should succeed");
        }
    }

    #[test]
    fn port_path_appends_port_extension() {
        let p = port_path(&PathBuf::from("/tmp/daemon.sock"));
        assert_eq!(p, PathBuf::from("/tmp/daemon.port"));
    }

    #[test]
    fn token_path_appends_token_extension() {
        let p = token_path(&PathBuf::from("/tmp/daemon.sock"));
        assert_eq!(p, PathBuf::from("/tmp/daemon.token"));
    }

    #[test]
    fn generates_non_empty_token() {
        let token = generate_token();
        assert!(!token.is_empty());
        assert!(token.starts_with("lode-ipc-token-"));
    }

    #[test]
    fn authenticate_valid_token_succeeds() {
        let token = "test-token";
        assert!(authenticate(token, token).is_ok());
    }

    #[test]
    fn authenticate_invalid_token_fails() {
        assert!(authenticate("wrong", "correct").is_err());
    }

    #[test]
    fn process_ipc_line_with_json_and_token() {
        let (tx, _rx) = mpsc::channel(1);
        let control = DaemonControl::new(tx);
        let token = "test-token-123";
        let line = r#"{"command":"status","token":"test-token-123"}"#;
        let response = process_ipc_line(line, token, &control).unwrap();
        assert!(response.success);
    }

    #[test]
    fn process_ipc_line_without_token_fails() {
        let (tx, _rx) = mpsc::channel(1);
        let control = DaemonControl::new(tx);
        let token = "server-token";
        let line = "status";
        let response = process_ipc_line(line, token, &control).unwrap();
        assert!(!response.success);
        assert!(response.message.contains("Authentication failed"));
    }

    #[test]
    fn process_ipc_line_with_plain_text_token_succeeds() {
        let (tx, _rx) = mpsc::channel(1);
        let control = DaemonControl::new(tx);
        let token = "server-token";
        let line = "status server-token";
        let response = process_ipc_line(line, token, &control).unwrap();
        assert!(response.success);
    }

    #[test]
    fn process_ipc_line_with_wrong_token_fails() {
        let (tx, _rx) = mpsc::channel(1);
        let control = DaemonControl::new(tx);
        let token = "server-token";
        let line = r#"{"command":"stop","token":"wrong-token"}"#;
        let response = process_ipc_line(line, token, &control).unwrap();
        assert!(!response.success);
        assert!(response.message.contains("Authentication failed"));
    }

    #[test]
    fn parse_json_request_valid() {
        let input = r#"{"command":"status","token":"abc"}"#;
        let req = parse_json_request(input).unwrap();
        assert_eq!(req.command, "status");
        assert_eq!(req.token, Some("abc".to_string()));
    }

    #[test]
    fn parse_json_request_minimal() {
        let input = r#"{"command":"stop"}"#;
        let req = parse_json_request(input).unwrap();
        assert_eq!(req.command, "stop");
        assert_eq!(req.token, None);
    }

    #[test]
    fn parse_json_request_invalid() {
        assert!(parse_json_request("not json").is_err());
    }

    #[test]
    fn read_token_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("daemon.sock");
        let expected = "lode-ipc-token-123-abc";
        std::fs::write(&token_path(&sock_path), expected).unwrap();
        let actual = read_token(&sock_path).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_token_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("nonexistent.sock");
        assert!(read_token(&sock_path).is_err());
    }
}
