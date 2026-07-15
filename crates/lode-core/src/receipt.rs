use std::fs;
use std::path::PathBuf;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandReceipt {
    pub schema_version: u32,
    pub receipt_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub status: ReceiptStatus,
    pub steps: Vec<ReceiptStep>,
    pub result: ReceiptResult,
    pub changed_files: Vec<String>,
    pub generated_assets: Vec<String>,
    pub git_state: Option<GitState>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    pub resumption_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiptStatus {
    Running,
    Completed,
    Failed(String),
    Resumable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptStep {
    pub id: String,
    pub description: String,
    pub status: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptResult {
    pub exit_code: i32,
    pub summary: String,
    pub next_actions: Vec<NextAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAction {
    pub command: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    pub sha: String,
    pub branch: String,
    pub message: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub dirty: bool,
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("T{}", d.as_secs())
}

fn short_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("receipt_{:016x}", nanos)
}

impl CommandReceipt {
    pub fn new(command: &str, args: &[String]) -> Self {
        Self {
            schema_version: 1,
            receipt_id: short_id(),
            command: command.to_string(),
            args: args.to_vec(),
            started_at: now_iso(),
            completed_at: None,
            duration_ms: None,
            status: ReceiptStatus::Running,
            steps: Vec::new(),
            result: ReceiptResult {
                exit_code: 0,
                summary: String::new(),
                next_actions: Vec::new(),
            },
            changed_files: Vec::new(),
            generated_assets: Vec::new(),
            git_state: None,
            warnings: Vec::new(),
            error: None,
            resumption_point: None,
        }
    }

    pub fn add_step(&mut self, id: &str, description: &str) {
        self.steps.push(ReceiptStep {
            id: id.to_string(),
            description: description.to_string(),
            status: "pending".to_string(),
            duration_ms: None,
        });
    }

    pub fn complete_step(&mut self, id: &str, status: &str, duration_ms: u64) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == id) {
            step.status = status.to_string();
            step.duration_ms = Some(duration_ms);
        }
    }

    pub fn complete(&mut self, exit_code: i32, summary: &str) {
        self.completed_at = Some(now_iso());
        self.status = ReceiptStatus::Completed;
        self.result.exit_code = exit_code;
        self.result.summary = summary.to_string();
        self.duration_ms = None;
    }

    pub fn capture_git_state(project_dir: &Utf8Path) -> Option<GitState> {
        let git_dir = project_dir.join(".git");
        if !git_dir.exists() {
            return None;
        }
        let sha = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(project_dir)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let branch = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(project_dir)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let dirty = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(project_dir)
            .output()
            .ok()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        Some(GitState {
            sha,
            branch,
            message: String::new(),
            files_changed: 0,
            insertions: 0,
            deletions: 0,
            dirty,
        })
    }

    pub fn save(&self, project_dir: &Utf8Path, receipts_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let root = ValidatedRoot::new(project_dir)?;
        root.create_dir_all(receipts_dir.as_str())?;
        let path = receipts_dir.join(format!("{}.json", self.receipt_id));
        let json =
            serde_json::to_string_pretty(self).map_err(|e| LodeError::Message(e.to_string()))?;
        root.write_atomic(
            receipts_dir
                .join(format!("{}.json", self.receipt_id))
                .as_str(),
            json,
        )?;
        Ok(path)
    }

    pub fn load(path: &Utf8Path) -> Result<Self> {
        let raw = fs::read_to_string(path).map_err(|e| LodeError::Io {
            path: PathBuf::from(path.as_str()),
            source: e,
        })?;
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))
    }

    pub fn list(receipts_dir: &Utf8Path) -> Result<Vec<String>> {
        if !receipts_dir.exists() {
            return Ok(Vec::new());
        }
        let mut receipts = Vec::new();
        for entry in fs::read_dir(receipts_dir).map_err(|e| LodeError::Io {
            path: PathBuf::from(receipts_dir.as_str()),
            source: e,
        })? {
            let entry = entry.map_err(|e| LodeError::Io {
                path: PathBuf::from(receipts_dir.as_str()),
                source: e,
            })?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    receipts.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        receipts.sort();
        receipts.reverse();
        Ok(receipts)
    }
}
