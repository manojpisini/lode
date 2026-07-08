#[cfg(unix)]
use std::path::PathBuf;

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
        let socket_path = socket_path(project_name);
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
pub struct DaemonIpc;

#[cfg(not(unix))]
impl DaemonIpc {
    pub fn connect(_project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Daemon IPC is not supported on this platform".into())
    }

    pub fn read_event(&mut self) -> Option<DaemonEvent> {
        None
    }
}

#[cfg(unix)]
fn socket_path(project_name: &str) -> PathBuf {
    let temp = std::env::temp_dir();
    let _ =
        lode_core::ValidatedRoot::new(&temp).and_then(|root| root.create_dir_all("lode-daemon"));
    temp.join("lode-daemon")
        .join(format!("{}.sock", project_name))
}
