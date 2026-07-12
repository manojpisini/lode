use std::io::BufRead;
#[cfg(unix)]
use std::path::PathBuf;

use lode_core::ipc::socket_port;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum DaemonEvent {
    FileChanged { path: String },
    ConventionViolation { file: String, rule: String },
    SecretFound { file: String, line: u32 },
    BuildStarted,
    BuildFinished { success: bool },
    TestRan { passed: u32, failed: u32 },
    LintReported { errors: u32, warnings: u32 },
    HealthChecked { score: u8 },
}

#[cfg(unix)]
pub struct DaemonIpc {
    reader: std::io::BufReader<std::os::unix::net::UnixStream>,
}

#[cfg(unix)]
impl DaemonIpc {
    pub fn connect(project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_path = daemon_socket_path().or_else(|| legacy_socket_path(project_name));
        let stream = std::os::unix::net::UnixStream::connect(&socket_path)?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            reader: std::io::BufReader::new(stream),
        })
    }

    pub fn read_event(&mut self) -> Option<DaemonEvent> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => serde_json::from_str(&line).ok(),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }
}

#[cfg(not(unix))]
pub struct DaemonIpc {
    reader: std::io::BufReader<std::net::TcpStream>,
}

#[cfg(not(unix))]
impl DaemonIpc {
    pub fn connect(_project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_path = std::path::PathBuf::from(".lode")
            .join("daemon")
            .join("daemon.sock");
        let port_path = socket_path.with_extension("port");
        let port = std::fs::read_to_string(&port_path)
            .ok()
            .and_then(|raw| raw.trim().parse::<u16>().ok())
            .unwrap_or_else(|| socket_port(&socket_path));
        let stream = std::net::TcpStream::connect(("127.0.0.1", port))?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            reader: std::io::BufReader::new(stream),
        })
    }

    pub fn read_event(&mut self) -> Option<DaemonEvent> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => serde_json::from_str(&line).ok(),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }
}

#[cfg(unix)]
fn daemon_socket_path() -> Option<PathBuf> {
    let path = PathBuf::from(".lode").join("daemon").join("daemon.sock");
    path.exists().then_some(path)
}

#[cfg(unix)]
fn legacy_socket_path(project_name: &str) -> PathBuf {
    // Sanitize project name to prevent path traversal
    let safe_name: String = project_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(128)
        .collect();
    let temp = std::env::temp_dir();
    let _ =
        lode_core::ValidatedRoot::new(&temp).and_then(|root| root.create_dir_all("lode-daemon"));
    temp.join("lode-daemon").join(format!("{}.sock", safe_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_event_deserializes() {
        let event: DaemonEvent =
            serde_json::from_str(r#"{"FileChanged":{"path":"src/main.rs"}}"#).unwrap();
        assert!(matches!(event, DaemonEvent::FileChanged { path } if path == "src/main.rs"));
    }

    #[cfg(not(unix))]
    #[test]
    fn socket_port_is_stable() {
        let path = std::path::Path::new(".lode/daemon/daemon.sock");
        assert_eq!(socket_port(path), socket_port(path));
    }
}
