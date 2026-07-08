use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("IPC server failed: {0}")]
    ServerFailed(String),
    #[error("Command parse error: {0}")]
    ParseError(String),
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
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            running: false,
        }
    }

    pub async fn start(&mut self) -> Result<(), IpcError> {
        if self.running {
            return Err(IpcError::ServerFailed("Already running".to_string()));
        }

        if self.socket_path.exists() {
            tokio::fs::remove_file(&self.socket_path).await?;
        }

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

        self.running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

pub fn handle_command(command: &IpcCommand) -> IpcResponse {
    match command {
        IpcCommand::Status => {
            IpcResponse::ok("Daemon is running").with_data(serde_json::json!({"status": "running"}))
        }
        IpcCommand::Stop => IpcResponse::ok("Stop requested"),
        IpcCommand::Pause => IpcResponse::ok("Paused"),
        IpcCommand::Resume => IpcResponse::ok("Resumed"),
        IpcCommand::ListWatchers => {
            IpcResponse::ok("Watchers listed").with_data(serde_json::json!({"watchers": []}))
        }
        IpcCommand::Reload => IpcResponse::ok("Configuration reloaded"),
    }
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

pub async fn start_ipc(socket_path: PathBuf) -> Result<IpcServer, IpcError> {
    let mut server = IpcServer::new(socket_path);
    server.start().await?;
    Ok(server)
}

#[cfg(unix)]
pub async fn run_ipc_listener(socket_path: PathBuf) -> Result<(), IpcError> {
    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }

    let listener = tokio::net::UnixListener::bind(&socket_path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    let line = line.trim();
                    if !line.is_empty() {
                        if let Ok(command) = parse_command(line) {
                            let _response = handle_command(&command);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }
}

#[cfg(not(unix))]
pub async fn run_ipc_listener(_socket_path: PathBuf) -> Result<(), IpcError> {
    Err(IpcError::ServerFailed(
        "IPC listener is not supported on this platform yet".to_string(),
    ))
}
