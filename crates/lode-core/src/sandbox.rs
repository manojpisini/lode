use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub sandbox_dir: PathBuf,
    pub files_written: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub temp_parent: PathBuf,
    pub timeout_secs: u64,
    pub max_output_bytes: u64,
    pub inherit_env: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            temp_parent: std::env::temp_dir(),
            timeout_secs: 30,
            max_output_bytes: 1_048_576,
            inherit_env: false,
        }
    }
}

pub fn create_sandbox(config: &SandboxConfig) -> Result<PathBuf> {
    let sandbox_dir = config
        .temp_parent
        .join(format!("lode-sandbox-{}", uuid_lite()));
    fs::create_dir_all(&sandbox_dir).map_err(|source| crate::LodeError::Io {
        path: sandbox_dir.clone(),
        source,
    })?;
    Ok(sandbox_dir)
}

pub fn run_in_sandbox(
    config: &SandboxConfig,
    command: &str,
    args: &[&str],
    files: &[(String, String)],
) -> Result<SandboxResult> {
    let sandbox_dir = create_sandbox(config)?;

    for (name, content) in files {
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(crate::LodeError::Message(format!(
                "sandbox file name contains path separator or traversal: {name}"
            )));
        }
        let file_path = sandbox_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|source| crate::LodeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&file_path, content).map_err(|source| crate::LodeError::Io {
            path: file_path.clone(),
            source,
        })?;
    }

    crate::process::validate_program(command)?;

    let start = Instant::now();
    let output = std::process::Command::new(command)
        .args(args)
        .current_dir(&sandbox_dir)
        .env_if(config.inherit_env)
        .output();
    let duration = start.elapsed();

    let (exit_code, stdout, stderr) = match output {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let stdout = truncate_bytes(&out.stdout, config.max_output_bytes);
            let stderr = truncate_bytes(&out.stderr, config.max_output_bytes);
            (code, stdout, stderr)
        }
        Err(e) => (-1, String::new(), format!("failed to execute: {e}")),
    };

    let files_written = list_files(&sandbox_dir);

    cleanup(&sandbox_dir);

    Ok(SandboxResult {
        exit_code,
        stdout,
        stderr,
        duration_ms: duration.as_millis() as u64,
        sandbox_dir,
        files_written,
    })
}

fn uuid_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_nanos();
    format!("{:x}", nanos)
}

fn truncate_bytes(data: &[u8], max: u64) -> String {
    let len = (data.len() as u64).min(max) as usize;
    String::from_utf8_lossy(&data[..len]).to_string()
}

fn list_files(dir: &PathBuf) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(dir) {
                    files.push(rel.display().to_string());
                }
            }
        }
    }
    files
}

fn cleanup(dir: &PathBuf) {
    fn remove_all(path: &PathBuf) {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    remove_all(&entry.path());
                }
            }
            let _ = fs::remove_dir(path);
        } else {
            let _ = fs::remove_file(path);
        }
    }
    remove_all(dir);
}

trait EnvIf {
    fn env_if(self, inherit: bool) -> Self;
}

impl EnvIf for &mut std::process::Command {
    fn env_if(self, inherit: bool) -> Self {
        if !inherit {
            self.env_clear()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_bytes_short() {
        let s = truncate_bytes(b"hello", 100);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_truncate_bytes_long() {
        let s = truncate_bytes(b"hello world", 5);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_sandbox_create_and_cleanup() {
        let config = SandboxConfig::default();
        let dir = create_sandbox(&config).unwrap();
        assert!(dir.exists());
        cleanup(&dir);
        assert!(!dir.exists());
    }

    #[test]
    fn test_sandbox_run_echo_inherit_env() {
        let config = SandboxConfig {
            inherit_env: true,
            ..SandboxConfig::default()
        };
        #[cfg(windows)]
        let result = run_in_sandbox(&config, "cmd.exe", &["/c", "echo", "hello"], &[]).unwrap();
        #[cfg(not(windows))]
        let result = run_in_sandbox(&config, "echo", &["hello"], &[]).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_sandbox_rejects_path_like_command() {
        let config = SandboxConfig::default();
        let result = run_in_sandbox(&config, "../sh", &["-c", "echo hi"], &[]);
        assert!(result.is_err(), "path-like command should be rejected");
    }

    #[test]
    fn test_sandbox_rejects_file_traversal() {
        let config = SandboxConfig::default();
        let result = run_in_sandbox(
            &config,
            "cmd.exe",
            &["/c", "echo", "hello"],
            &[("../../escape.txt".to_string(), "pwned".to_string())],
        );
        assert!(
            result.is_err(),
            "file name with path traversal should be rejected"
        );
    }
}
